
use common::*;
use crate::util;
use na;
use std::collections::BTreeMap;
use std::time::Duration;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{ CanvasRenderingContext2d, Node, FillRule };
use super::value_button::ValueButton;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalService, IntervalTask, };
use yew::virtual_dom::vnode::VNode;
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };

const REALTIME_USER_POLL_RATE: Duration = Duration::from_millis(1000);

const MAP_WIDTH: u32 = 800;
const MAP_HEIGHT: u32 = 800;
const MAP_SCALE: f64 = MAP_WIDTH as f64 / 4.0;

pub enum Msg {
    RenderMap,
    ViewDistance(MacAddress),

    RequestRealtimeUser,

    ResponseRealtimeUser(util::Response<Vec<common::TrackedUser>>),
}

pub struct MapViewComponent {
    context: CanvasRenderingContext2d,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    interval_service: Option<IntervalService>,
    interval_service_task: Option<IntervalTask>,
    map_canvas: CanvasElement,
    self_link: ComponentLink<MapViewComponent>,
    show_distance: Option<MacAddress>,
    users: BTreeMap<MacAddress, Box<common::TrackedUser>>,
}

impl MapViewComponent {
    fn start_service(&mut self) {
        let mut interval_service = IntervalService::new();
        self.interval_service_task = Some(
            interval_service.spawn(REALTIME_USER_POLL_RATE, self.self_link.send_back(|_| Msg::RequestRealtimeUser))
        );
        self.interval_service = Some(interval_service);
    }

    fn end_service(&mut self) {
        self.interval_service = None;
        self.interval_service_task = None;
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct MapViewProps {
    pub emergency: bool,
}

fn screen_space(x: f64, y: f64) -> na::Vector2<f64> {
    na::Vector2::new(x, MAP_HEIGHT as f64 - y)
}

impl MapViewComponent {
    fn clear_map(&self) {
        // clear the canvas and draw a border

        self.context.set_line_dash(vec![]);
        self.context.clear_rect(0.0, 0.0, self.map_canvas.width().into(), self.map_canvas.height().into());
        self.context.stroke_rect(0.0, 0.0, self.map_canvas.width().into(), self.map_canvas.height().into());

        self.context.save();
        self.context.set_line_dash(vec![5.0, 15.0]);
        // vertical gridlines
        for i in (MAP_SCALE as u32..MAP_WIDTH as u32).step_by(MAP_SCALE as usize) {
            let pos0 = screen_space(i as f64, MAP_HEIGHT as f64);
            let pos1 = screen_space(i as f64, 0.0);
            self.context.begin_path();
            self.context.move_to(pos0.x, pos0.y);
            self.context.line_to(pos1.x, pos1.y);
            self.context.stroke();
        }
        // horizontal gridlines
        for i in (MAP_SCALE as u32..MAP_HEIGHT as u32).step_by(MAP_SCALE as usize) {
            let pos0 = screen_space(MAP_WIDTH as f64, i as f64);
            let pos1 = screen_space(0.0, i as f64);
            self.context.begin_path();
            self.context.move_to(pos0.x, pos0.y);
            self.context.line_to(pos1.x, pos1.y);
            self.context.stroke();
        }
        self.context.restore();

        let text_adjustment = 10.0;
        // x axis
        for i in 0..(MAP_WIDTH / MAP_SCALE as u32) {
            let pos = screen_space(i as f64 * MAP_SCALE + text_adjustment, text_adjustment);
            self.context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
        }
        // y axis
        // skip 0 because it was rendered by the y axis.
        for i in 1..(MAP_HEIGHT / MAP_SCALE as u32) {
            let pos = screen_space(text_adjustment, i as f64 * MAP_SCALE + text_adjustment);
            self.context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
        }
    }
}

fn get_context(canvas: &CanvasElement) -> CanvasRenderingContext2d {
    unsafe {
        js! (
            return @{canvas}.getContext("2d");
        ).into_reference_unchecked().unwrap()
    }
}

impl Component for MapViewComponent {
    type Message = Msg;
    type Properties = MapViewProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let canvas: CanvasElement = unsafe {
            js! (
                let c = document.createElement("canvas");
                c.setAttribute("id", "map_canvas");
                return c;
            ).into_reference_unchecked().unwrap()
        };
        canvas.set_width(MAP_WIDTH);
        canvas.set_height(MAP_HEIGHT);
        let context = get_context(&canvas);

        let mut result = MapViewComponent {
            context: context,
            emergency: props.emergency,
            fetch_service: FetchService::new(),
            fetch_task: None,
            interval_service: None,
            interval_service_task: None,
            map_canvas: canvas,
            self_link: link,
            show_distance: None,
            users: BTreeMap::new(),
        };

        result.clear_map();
        if props.emergency {
            result.start_service();
        }
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RenderMap => {
                self.context.save();
                for (_tag_mac, user) in self.users.iter() {
                    let user_pos = screen_space(
                        user.coordinates.x as f64 * MAP_SCALE,
                        user.coordinates.y as f64 * MAP_SCALE,
                    );

                    for beacon_source in &user.beacon_sources {
                        let beacon_loc = screen_space(
                            beacon_source.location.x * MAP_SCALE,
                            beacon_source.location.y * MAP_SCALE,
                        );
                        self.context.set_fill_style_color("#0000FFFF");
                        self.context.fill_rect(beacon_loc.x, beacon_loc.y - 30.0, 30.0, 30.0);
                        self.context.set_fill_style_color("#000000FF");
                        self.context.fill_rect(user_pos.x, user_pos.y, 20.0, 20.0);
                        match &self.show_distance {
                            Some(tag_mac) if tag_mac == &user.mac_address => {
                                self.context.set_fill_style_color("#00000034");
                                self.context.begin_path();
                                self.context.arc(beacon_loc.x, beacon_loc.y, beacon_source.distance_to_tag * MAP_SCALE, 0.0, std::f64::consts::PI * 2.0, true);
                                self.context.fill(FillRule::NonZero);
                            },
                            _ => { }
                        }
                    }
                }
                self.context.restore();

                return false;
            },
            Msg::ViewDistance(selected_tag_mac) => {
                match &self.show_distance {
                    Some(current_tag) => {
                        if current_tag == &selected_tag_mac {
                            self.show_distance = None;
                        } else {
                            self.show_distance = Some(selected_tag_mac);
                        }
                    },
                    None => {
                        self.show_distance = Some(selected_tag_mac);
                    }
                }
                return true;
            },
            Msg::RequestRealtimeUser => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &users_realtime_url(),
                    self.self_link,
                    Msg::ResponseRealtimeUser
                );

                return false;
            },
            Msg::ResponseRealtimeUser(response) => {
                self.clear_map();
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    if let Ok(data) = body {
                        for user in data.iter() {
                            match self.users.get_mut(&user.mac_address) {
                                Some(local_user_data) => {
                                    **local_user_data = user.clone();
                                },
                                None => {
                                    self.users.insert(user.mac_address.clone(), Box::new(user.clone()));
                                }

                            }
                        }
                    }
                } else {
                    Log!("response - failed to get realtime user data");
                }

                self.self_link.send_self(Msg::RenderMap);
                return true;
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        // do not overwrite the canvas or context.
        self.emergency = props.emergency;

        if self.emergency {
            self.start_service();
        } else {
            self.end_service();
        }
        true
    }
}

impl Renderable<MapViewComponent> for MapViewComponent {
    fn view(&self) -> Html<Self> {
        let mut render_distance_buttons = self.users.iter().map(|(user_mac, _user)| {
            let set_border = match &self.show_distance {
                Some(selected) => selected == user_mac,
                None => false,
            };
            html! {
                <ValueButton<String>
                    on_click=|value: String| Msg::ViewDistance(MacAddress::parse_str(&value).unwrap()),
                    border=set_border,
                    value={user_mac.to_hex_string()}
                />
            }
        });

        html! {
            <div>
                <div>
                    {
                        if self.users.len() > 0 {
                            "View Tag Distance Values: "
                        } else {
                            ""
                        }
                    }
                    { for render_distance_buttons }
                </div>
                { VNode::VRef(Node::from(self.map_canvas.to_owned()).to_owned()) }
            </div>
        }
    }
}
