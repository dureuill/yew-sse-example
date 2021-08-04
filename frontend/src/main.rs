use yew::{Component, html};

mod event_send {
    use yew::{prelude::*, services::fetch};

    pub struct Model {
        link: ComponentLink<Self>,
        current_task: Option<fetch::FetchTask>,
        message: String,
        last_error: Option<anyhow::Error>,
    }

    impl Model {
        fn view_last_error(&self) -> Html {
            if let Some(err) = self.last_error.as_ref() {
                html! {
                    <p>{format!("Sending failed: {}", err)}</p>
                }
            } else {
                html! {<p></p>}
            }
        }
    }

    pub enum Msg {
        Response(Result<(), anyhow::Error>),
        UpdateMessage(String),
        SendMessage(String),
    }

    impl Component for Model {
        type Message = Msg;
        type Properties = ();

        fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
            Self {
                current_task: None,
                message: Default::default(),
                link,
                last_error: None,
            }
        }

        fn update(&mut self, msg: Self::Message) -> ShouldRender {
            match msg {
                Msg::Response(res) => {
                    if let Err(err) = res {
                        self.last_error = Some(err)
                    } else {
                        self.last_error = None;
                        self.message.clear();
                        self.current_task = None;
                    }
                    true
                }
                Msg::SendMessage(msg) => {
                    let post_request = fetch::Request::post("http://localhost:8000/msg")
                        .body(Ok(msg))
                        .expect("Could not build that request.");
                    let callback = self.link.callback(
                        |response: fetch::Response<Result<String, anyhow::Error>>| {
                            Msg::Response(response.into_body().map(|_| ()))
                        },
                    );
                    let mut fetch_service = fetch::FetchService::new();
                    // 3. pass the request and callback to the fetch service
                    let task = fetch_service
                        .fetch(post_request, callback)
                        .expect("failed to start request");
                    // 4. store the task so it isn't canceled immediately
                    self.current_task = Some(task);
                    // we want to redraw so that the page displays a 'fetching...' message to the user
                    // so return 'true'
                    self.message.clear();
                    true
                }
                Msg::UpdateMessage(msg) => {
                    self.message = msg;
                    true
                }
            }
        }

        fn change(&mut self, _props: Self::Properties) -> ShouldRender {
            // Should only return "true" if new properties are different to
            // previously received properties.
            // This component has no properties so we will always return "false".
            false
        }

        fn view(&self) -> Html {
            if let Some(_) = self.current_task {
                html! {
                    <div>{"Sending message..."}</div>
                }
            } else {
                let msg = self.message.clone();
                html! {
                    <div>
                        <label for="msg">{"Message:"}</label>
                        <input type="text" id="msg" name="msg" placeholder="Send message" value={msg.clone()} oninput={self.link.callback(|e : InputData| Msg::UpdateMessage(e.value))}/>
                        <input type="submit" value="Send" disabled={msg.is_empty()} onclick={self.link.callback(move |_| Msg::SendMessage(msg.clone()))}/>
                        {self.view_last_error()}
                    </div>
                }
            }
        }
    }
}

mod event_stream {
    use yew::prelude::*;
    use yew_sse::services::{
        self,
        sse::{EventSourceTask, EventSourceUpdate},
    };

    pub struct Event {
        title: String,
        description: String,
    }

    pub enum Msg {
        PushEvent(Event),
        ReceivedUpdate(EventSourceUpdate),
    }

    pub struct Model {
        // `ComponentLink` is like a reference to a component.
        // It can be used to send messages to the component
        events: Vec<Event>,
        task: Result<EventSourceTask, String>,
        last_event: Option<EventSourceUpdate>,
    }

    fn render_event_source_update(update: &EventSourceUpdate) -> Html {
        match update {
            EventSourceUpdate::Error => html! { <pre>{"error"}</pre>},
            EventSourceUpdate::Open => html! { <pre>{"open"}</pre>},
        }
    }

    impl Component for Model {
        type Message = Msg;
        type Properties = ();

        fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
            let service = services::EventSourceService::new();
            let callback = link.callback(|(msg0, msg1)| {
                Msg::PushEvent(Event {
                    title: msg0,
                    description: msg1,
                })
            });
            let updates = link.callback(|update| Msg::ReceivedUpdate(update));
            let task = service
                .open("http://localhost:8000/events", callback, updates)
                .map_err(Into::into);
            Self {
                events: vec![],
                task,
                last_event: None,
            }
        }

        fn update(&mut self, msg: Self::Message) -> ShouldRender {
            match msg {
                Msg::PushEvent(event) => {
                    self.events.push(event);
                    // the value has changed so we need to
                    // re-render for it to appear on the page
                    true
                }
                Msg::ReceivedUpdate(event) => {
                    self.last_event = Some(event);
                    true
                }
            }
        }

        fn change(&mut self, _props: Self::Properties) -> ShouldRender {
            // Should only return "true" if new properties are different to
            // previously received properties.
            // This component has no properties so we will always return "false".
            false
        }

        fn view(&self) -> Html {
            let last_event = match &self.last_event {
                Some(event) => html! {
                    {render_event_source_update(event)}
                },
                None => html! {<pre>{"No last event"}</pre>},
            };
            match &self.task {
                Ok(_) => {
                    html! {
                        <div>
                            <h1>{"Events"}</h1>
                            <ul>
                                { for self.events.iter().map(render_event_item) }
                            </ul>
                            {last_event}
                        </div>
                    }
                }
                Err(err) => {
                    html! { <div>{format!("Error: {}", err)}</div> }
                }
            }
        }
    }

    fn render_event_item(event: &Event) -> Html {
        html! {
            <li>
            {format!("{} => {}", event.title, event.description)}
            </li>
        }
    }
}

struct Model {}
impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> yew::ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> yew::ShouldRender {
        false
    }

    fn view(&self) -> yew::Html {
        html! {
            <div>
            <div><event_send::Model/></div>
            <div><event_stream::Model/></div>
            </div>
        }
    }

}

fn main() {
    yew::start_app::<Model>();
}
