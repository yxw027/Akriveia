use common::*;
use crate::util;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };

pub enum Msg {
    AddAnotherMap,
    InputBound(usize, String),
    InputName(String),
    InputNote(String),

    RequestAddUpdateMap,
    RequestGetMap(i32),
    RequestGetBeaconsForMap(i32),

    ResponseAddMap(util::Response<Map>),
    ResponseGetBeaconsForMap(util::Response<Vec<Beacon>>),
    ResponseGetMap(util::Response<Option<Map>>),
    ResponseUpdateMap(util::Response<Map>),
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    pub map: Map,
    pub error_messages: Vec<String>,
    pub attached_beacons: Vec<Beacon>,
    pub id: Option<i32>,
    pub raw_bound0: String,
    pub raw_bound1: String,
    pub success_message: Option<String>,
}

impl Data {
    fn new() -> Data {
        Data {
            map: Map::new(),
            error_messages: Vec::new(),
            attached_beacons: Vec::new(),
            id: None,
            raw_bound0: "0".to_string(),
            raw_bound1: "0".to_string(),
            success_message: None,
        }
    }

    /*fn validate(&mut self) -> bool {
        let mut success = match MacAddress::parse_str(&self.raw_mac) {
            Ok(m) => {
                self.beacon.mac_address = m;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse mac address: {}", e));
                false
            },
        };

        success = success && match self.raw_coord0.parse::<f64>() {
            Ok(coord) => {
                self.beacon.coordinates[0] = coord;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse x coordinate: {}", e));
                false
            },
        };

        success = success && match self.raw_coord1.parse::<f64>() {
            Ok(coord) => {
                self.beacon.coordinates[1] = coord;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse y coordinate: {}", e));
                false
            },
        };

        success
    }*/
}

pub struct MapAddUpdate {
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    get_fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
}

#[derive(Clone, Default, PartialEq)]
pub struct MapAddUpdateProps {
    pub opt_id: Option<i32>,
}

impl Component for MapAddUpdate {
    type Message = Msg;
    type Properties = MapAddUpdateProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        if let Some(id) = props.opt_id {
            link.send_self(Msg::RequestGetMap(id));
            link.send_self(Msg::RequestGetBeaconsForMap(id));
        }
        let mut result = MapAddUpdate {
            data: Data::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            get_fetch_task: None,
            self_link: link,
        };
        result.data.id = props.opt_id;
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddAnotherMap => {
                self.data = Data::new();
            }
            Msg::InputName(name) => {
                self.data.map.name = name;
            },
            Msg::InputNote(note) => {
                self.data.map.note = Some(note);
            },
            Msg::InputBound(index, value) => {
                match index {
                    0 => { self.data.raw_bound0 = value; },
                    1 => { self.data.raw_bound1 = value; },
                    _ => panic!("invalid coordinate index specified"),
                };
            },
            Msg::RequestGetBeaconsForMap(id) => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &beacons_for_map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeaconsForMap
                );
            },
            Msg::RequestGetMap(id) => {
                self.get_fetch_task = get_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetMap
                );
            },
            Msg::RequestAddUpdateMap => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;

                //let success = self.data.validate();

                match self.data.id {
                    Some(id) => {
                        //ensure the id does not mismatch.
                        self.data.map.id = id;

                        self.fetch_task = put_request!(
                            self.fetch_service,
                            &map_url(&self.data.map.id.to_string()),
                            self.data.map,
                            self.self_link,
                            Msg::ResponseUpdateMap
                        );
                    },
                    None => {
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &map_url(""),
                            self.data.map,
                            self.self_link,
                            Msg::ResponseAddMap
                        );
                    },
                }
            },
            Msg::ResponseGetBeaconsForMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.attached_beacons = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to obtain available floors list, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to obtain available floors list".to_string());
                }
            },
            Msg::ResponseUpdateMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.success_message = Some("successfully updated map".to_string());
                            self.data.map = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to update map, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to update map".to_string());
                }
            },
            Msg::ResponseGetMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.map = result.unwrap_or(Map::new());
                            self.data.raw_bound0 = self.data.map.bounds[0].to_string();
                            self.data.raw_bound1 = self.data.map.bounds[1].to_string();
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to find map, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to find map".to_string());
                }
            },
            Msg::ResponseAddMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.success_message = Some("successfully added map".to_string());
                            self.data.map = result;
                            self.data.id = Some(self.data.map.id);
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to add map, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to add map".to_string());
                }
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.data.id = props.opt_id;
        true
    }
}

impl Renderable<MapAddUpdate> for MapAddUpdate {
    fn view(&self) -> Html<Self> {
        let submit_name = match self.data.id {
            Some(_id) => "Update Map",
            None => "Add Map",
        };
        let title_name = match self.data.id {
            Some(_id) => "Map Update",
            None => "Map Add",
        };

        let add_another_map = match &self.data.id {
            Some(_) => {
                html! {
                    <button onclick=|_| Msg::AddAnotherMap,>{ "Add Another" }</button>
                }
            },
            None => {
                html! { <></> }
            },
        };

        /*let mut attached_beacons = self.data.attached_beacons.iter().cloned().map(|beacon| {
            html! {
                <option
                    onclick=|_| Msg::InputFloorName(floor_id),
                    disabled={ floor_id == chosen_floor_id },
                >
                    { &beacon.name }
                </option>
            }
        });*/

        let mut errors = self.data.error_messages.iter().cloned().map(|msg| {
            html! {
                <p>{msg}</p>
            }
        });

        let note = self.data.map.note.clone().unwrap_or(String::new());

        html! {
            <>
                <p>{ title_name }</p>
                {
                    match &self.data.success_message {
                        Some(msg) => { format!("Success: {}", msg) },
                        None => { String::new() },
                    }
                }
                { if self.data.error_messages.len() > 0 { "Failure: " } else { "" } }
                { for errors }
                <div/>
                <table>
                    <tr>
                        <td>{ "Name: " }</td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.map.name,
                                oninput=|e| Msg::InputName(e.value),
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Bounds: " }</td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.raw_bound0,
                                oninput=|e| Msg::InputBound(0, e.value),
                            />
                        </td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.raw_bound1,
                                oninput=|e| Msg::InputBound(1, e.value),
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Note: " }</td>
                        <td>
                            <textarea
                                rows=5,
                                value=note,
                                oninput=|e| Msg::InputNote(e.value),
                            />
                        </td>
                    </tr>
                </table>
                <button onclick=|_| Msg::RequestAddUpdateMap,>{ submit_name }</button>
                { add_another_map }
            </>
        }
    }
}
