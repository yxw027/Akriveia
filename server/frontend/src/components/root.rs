
use common::*;
use crate::util::{ self, WebUserType, };
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::prelude::*;
use super::map_view::MapViewComponent;
use super::emergency_buttons::EmergencyButtons;
use super::diagnostics::Diagnostics;
use super::beacon_list::BeaconList;
use super::beacon_addupdate::BeaconAddUpdate;
use super::user_list::UserList;
use super::user_addupdate::UserAddUpdate;
use super::map_list::MapList;
use super::map_addupdate::MapAddUpdate;
use super::status::Status;
use super::login::Login;

#[derive(PartialEq)]
pub enum Page {
    BeaconAddUpdate(Option<i32>),
    BeaconList,
    UserAddUpdate(Option<i32>),
    UserList,
    Diagnostics,
    Status,
    Login(bool),
    MapView(Option<i32>),
    MapList,
    MapAddUpdate(Option<i32>),
}

pub struct RootComponent {
    user_type: WebUserType,
    current_page: Page,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<RootComponent>,
}

pub enum Msg {

    // page changes
    ChangePage(Page),
    ChangeWebUserType(WebUserType),

    // requests
    RequestPostEmergency(bool),
    RequestGetEmergency,

    // responses
    ResponsePostEmergency(util::Response<common::SystemCommandResponse>),
    ResponseGetEmergency(util::Response<common::SystemCommandResponse>),
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetEmergency);
        let root = RootComponent {
            user_type: WebUserType::Responder,
            current_page: Page::Login(true),
            emergency: false,
            fetch_service: FetchService::new(),
            fetch_task: None,
            link: link,
        };
        root
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangePage(page) => {
                self.current_page = page;
            },
            Msg::ChangeWebUserType(user_type) => {
                self.user_type = user_type;
            },

            // requests
            Msg::RequestPostEmergency(is_emergency) => {
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &system_emergency_url(),
                    SystemCommandResponse::new(is_emergency),
                    self.link,
                    Msg::ResponsePostEmergency
                );
            },
            Msg::RequestGetEmergency => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &system_emergency_url(),
                    self.link,
                    Msg::ResponseGetEmergency
                );
            },
            // responses
            Msg::ResponsePostEmergency(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::SystemCommandResponse { emergency }) => {
                            self.emergency = emergency;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to post start emergency");
                }
            },
            Msg::ResponseGetEmergency(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::SystemCommandResponse { emergency }) => {
                            self.emergency = emergency;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to request emergency status");
                }
            },
        }
        true
    }
}

impl Renderable<RootComponent> for RootComponent {
    fn view(&self) -> Html<Self> {
        match self.current_page {
            Page::Diagnostics => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <Diagnostics
                                emergency={self.emergency}
                            />
                        </div>
                    </div>
                }
            },
            Page::Status => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <Status
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::Login(auto_login) => {
                html! {
                    <div class="container-fluid">
                        <Login
                            change_page=|page| Msg::ChangePage(page),
                            change_user_type=|user_type| Msg::ChangeWebUserType(user_type),
                            auto_login = auto_login,
                        />
                    </div>
                }
            },
            Page::MapView(opt_id) => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <MapViewComponent
                                emergency={self.emergency},
                                opt_id=opt_id,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
            Page::BeaconList => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <BeaconList
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::BeaconAddUpdate(id) => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <BeaconAddUpdate
                                id=id,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
            Page::UserList => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <UserList
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::UserAddUpdate(id) => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <UserAddUpdate
                                id=id,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
            Page::MapList => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <MapList
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::MapAddUpdate(opt_id) => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <MapAddUpdate
                                opt_id=opt_id,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
        }
    }
}

impl RootComponent {
    fn navigation(&self) -> Html<Self> {
        let view_map = html! {
            <>
                <a
                    class = match self.current_page {
                        Page::MapView {..} => {"nav-link navBarText active"},
                        _ => {"nav-link navBarText"},
                    }
                    onclick=|_| Msg::ChangePage(Page::MapView(None)),                   
                    disabled={
                        match self.current_page {
                            Page::MapView { .. } => true,
                            _ => false,
                        }
                    },
                >
                    { "View Map" }
                </a>
            </>
        };

        let show_status = html! {
            <>
                <a 
                    class = match self.current_page {
                        Page::Status {..} => {"nav-link navBarText active"},
                        _ => {"nav-link navBarText"},
                    }
                    onclick=|_| Msg::ChangePage(Page::Status),
                    disabled={self.current_page == Page::Status},
                >
                    { "Status" }
                </a>
            </>
        };


        let select_user = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a 
                        class = match self.current_page {
                            Page::UserList => {"nav-link dropdown navBarText active"},
                            Page::UserAddUpdate{..} => {"nav-link dropdown navBarText active"},
                            _ => {"nav-link dropdown navBarText"},
                        } 
                        id="navbarDropdown", role="button" data-toggle="dropdown" aria-haspopup="true" aria-expanded="false",
                        onclick=|_| Msg::ChangePage(Page::UserList),
                        active={
                            match self.current_page {
                                Page::UserList => true,
                                Page::UserAddUpdate{..} => true,
                                _ => false,
                            }
                        },
                    >
                            { "User" }
                    </a>
                    <div class="dropdown-content" aria-labelledby="navbarDropdown">
                        <a
                            class="dropdown-item navBarText", 
                            onclick=|_| Msg::ChangePage(Page::UserList), 
                            disabled={
                                match self.current_page {
                                    Page::UserList => true,
                                    _ => false,
                                }
                            },
                        >
                                { "User List" }
                        </a>
                        <a
                            class="dropdown-item  navBarText",
                            onclick=|_| Msg::ChangePage(Page::UserAddUpdate(None)),
                            disabled={
                                match self.current_page {
                                    // match ignoring the fields
                                    Page::UserAddUpdate {..} => true,
                                    _=> false,
                                }
                            },
                        >
                            { "Add User" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! {
                <></>
            },
        };

        let select_beacon = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a 
                        class = match self.current_page {
                            Page::BeaconList => {"nav-link dropdown navBarText active"},
                            Page::BeaconAddUpdate{..} => {"nav-link dropdown navBarText active"},
                            _ => {"nav-link dropdown navBarText"},
                        } 
                        aria-haspopup="true" aria-expanded="false"
                        onclick=|_| Msg::ChangePage(Page::BeaconList),
                        active={
                            match self.current_page {
                                Page::BeaconList => true,
                                Page::BeaconAddUpdate {..} => true,
                                _ => false,
                            }
                        },
                    >
                        { "Beacons" }
                    </a>
                    <div class="dropdown-content">
                        <a 
                            class="dropdown-item navBarText", 
                            onclick=|_| Msg::ChangePage(Page::BeaconList), 
                            disabled={self.current_page == Page::BeaconList},>
                            { "Beacon List" }
                        </a>
                        <a
                            class="dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::BeaconAddUpdate(None)),
                            disabled={
                                match self.current_page {
                                    // match ignoring the fields
                                    Page::BeaconAddUpdate {..} => true,
                                    _ => false,
                                }
                            },
                        >
                            { "Add Beacon" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! {
                <></>
            }
        };

        let select_map = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a 
                        class = match self.current_page {
                            Page::MapList => {"nav-link dropdown navBarText active"},
                            Page::MapAddUpdate{..} => {"nav-link dropdown navBarText active"},
                            _ => {"nav-link dropdown navBarText"},
                        } 
                        aria-haspopup="true" aria-expanded="false"
                        onclick=|_| Msg::ChangePage(Page::MapList),
                        active={
                            match self.current_page {
                                Page::MapList => true,
                                Page::MapAddUpdate {..} => true,
                                _ => false,
                            }
                        },
                    >
                            { "Maps" }
                    </a>
                    <div class="dropdown-content">
                        <a 
                            class="dropdown-item navBarText"
                            onclick=|_| Msg::ChangePage(Page::MapList), 
                            disabled={self.current_page == Page::MapList},
                        >
                                { "Map List" }
                        </a>
                        <a
                            class="dropdown-item navBarText"
                            onclick=|_| Msg::ChangePage(Page::MapAddUpdate(None)),
                            disabled={
                                match self.current_page {
                                    // match ignoring the fields
                                    Page::MapAddUpdate {..} => true,
                                    _ => false,
                                }
                            },
                        >
                            { "Add Map" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! {
                <></>
            }
        };

        let diagnostics = match self.user_type {
            WebUserType::Admin => html! {
                <a 
                    class = match self.current_page {
                        Page::Diagnostics {..} => {"nav-link navBarText active"},
                        _ => {"nav-link navBarText"},
                    },
                    onclick=|_| Msg::ChangePage(Page::Diagnostics),
                    disabled={self.current_page == Page::Diagnostics},
                >
                    { "Diagnostics" }
                </a>
            },
            WebUserType::Responder => html! {
                <></>
            }
        };

        let login_type = match self.user_type {
            WebUserType::Admin => html!{
                <>
                    <button
                        class="btn btn-danger btn-sm nav-link logoutPlacement ml-auto",
                        onclick=|_| Msg::ChangePage(Page::Login(true)),
                        disabled={self.current_page == Page::Login(true)},
                    >
                        { "Logout" }
                    </button>
                    <a class="loginTypeHeader">{"ADMIN"}</a>
                </>
            },
            WebUserType::Responder => html!{
                <>
                    <button
                        class="btn btn-success btn-sm nav-link logoutPlacement ml-auto",
                        onclick=|_| Msg::ChangePage(Page::Login(false)),
                        disabled={self.current_page == Page::Login(false)},
                    >
                        { "Login" }
                    </button>
                    <a class="loginTypeHeader">{"FIRST RESPONDER"}</a>
                </>
            }
        };
        html! {
            <nav class="navbar navbar-expand-sm navbarColour">
                <a class="navbar-brand">
                    <img src="/images/icon.PNG" width="52" height="48" class="d-inline-block align-top" alt=""/>
                </a>
                <div class="navbarJustify">
                    <ul class="nav navbarText">
                        <li class="my-auto">
                            {view_map}
                        </li>
                        <li class="my-auto">
                            { diagnostics }
                        </li>
                        <li class="my-auto">
                            { show_status }
                        </li>
                        <li class="dropdown my-auto">
                            { select_beacon }
                        </li>
                        <li class="dropdown my-auto">
                            { select_user }
                        </li>
                        <li class="dropdown my-auto">
                            { select_map }
                        </li>
                    </ul>
                </div>
                {login_type}
            </nav>
        }
    }
}
