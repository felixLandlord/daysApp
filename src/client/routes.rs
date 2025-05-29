use crate::client::{
    components::{NavBar, PageNotFound},
    pages::{EmployeesPage, SchedulesPage, SettingsPage},
};
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(NavBar)]
    #[route("/")]
    SchedulesPage {},
    #[route("/employees")]
    EmployeesPage {},
    #[route("/settings")]
    SettingsPage {},
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}
