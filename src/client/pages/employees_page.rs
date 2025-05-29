use crate::client::components::SearchBar;
use crate::server::{
    db::{
        delete_employee, establish_connection, get_all_employees, insert_employee, update_employee,
    },
    schema::{Employee, Role, Sex, Weekday},
};

use dioxus::prelude::*;
//use rusqlite::Result;

const EMPLOYEES_CSS: Asset = asset!("/assets/styles/employees.css");
// const ADD_ICON: Asset = asset!("/assets/icons/add.svg");
const EDIT_ICON: Asset = asset!("/assets/icons/edit.svg");
const DELETE_ICON: Asset = asset!("/assets/icons/delete.svg");
const X_CLOSE_ICON: Asset = asset!("/assets/icons/x-close.svg");

#[derive(Debug, Clone, PartialEq)]
enum ModalType {
    None,
    Add,
    Edit(usize),
    Delete(usize),
    View(usize),
}

#[component]
pub fn EmployeesPage() -> Element {
    let mut employees = use_signal(|| match establish_connection() {
        Ok(conn) => match get_all_employees(&conn) {
            Ok(emps) => emps,
            Err(e) => {
                eprintln!("Failed to load employees: {}", e);
                Vec::new()
            }
        },
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            Vec::new()
        }
    });

    let mut search_query = use_signal(String::new);
    let mut modal_state = use_signal(|| ModalType::None);
    let mut current_employee = use_signal(|| Employee {
        id: 0,
        name: String::new(),
        sex: Sex::Male,
        role: Role::FullStackEngineer,
        required_days: 2,
        fixed_days: Vec::new(),
        is_nsp: false,
    });

    let mut next_id = use_signal(|| employees.read().iter().map(|e| e.id).max().unwrap_or(0) + 1);

    let filtered_employees = use_memo(move || {
        let query = search_query.read().to_lowercase();
        let source = &*employees.read();
        if query.is_empty() {
            source.clone()
        } else {
            source
                .iter()
                .filter(|emp| emp.name.to_lowercase().contains(&query))
                .cloned()
                .collect()
        }
    });

    let handle_search = move |query: String| {
        search_query.set(query);
    };

    let open_add_modal = move |_| {
        let new_id = *next_id.read();
        current_employee.set(Employee {
            id: new_id,
            name: String::new(),
            sex: Sex::Male,
            role: Role::FullStackEngineer,
            required_days: 2,
            fixed_days: Vec::new(),
            is_nsp: false,
        });
        modal_state.set(ModalType::Add);
    };

    let mut open_edit_modal = move |id: usize| {
        if let Some(emp) = employees.read().iter().find(|e| e.id == id).cloned() {
            current_employee.set(emp);
            modal_state.set(ModalType::Edit(id));
        }
    };

    let mut open_delete_modal = move |id: usize| {
        if let Some(emp) = employees.read().iter().find(|e| e.id == id).cloned() {
            current_employee.set(emp);
            modal_state.set(ModalType::Delete(id));
        }
    };

    let mut open_view_modal = move |id: usize| {
        if let Some(emp) = employees.read().iter().find(|e| e.id == id).cloned() {
            current_employee.set(emp);
            modal_state.set(ModalType::View(id));
        }
    };

    let close_modal = move |_| {
        modal_state.set(ModalType::None);
    };

    let handle_save = move |_| {
        let modal_type = modal_state.read().clone();
        let employee_data = current_employee.read().clone();

        match establish_connection() {
            Ok(conn) => match modal_type {
                ModalType::Add => {
                    if let Err(e) = insert_employee(&conn, &employee_data) {
                        eprintln!("Failed to insert employee: {}", e);
                    } else {
                        let new_id = *next_id.read();
                        let mut new_employee = employee_data.clone();
                        new_employee.id = new_id;
                        employees.write().push(new_employee);
                        next_id.set(new_id + 1);
                    }
                }
                ModalType::Edit(id) => {
                    if let Err(e) = update_employee(&conn, &employee_data) {
                        eprintln!("Failed to update employee: {}", e);
                    } else {
                        // Get a copy of the employees vector
                        let mut emp_list = employees.read().clone();

                        // Find the employee index in our copy
                        if let Some(i) = emp_list.iter().position(|e| e.id == id) {
                            // Update the employee in our copy
                            emp_list[i] = employee_data;

                            // Set the entire vector back to the signal
                            employees.set(emp_list);
                        }
                    }
                }
                _ => {}
            },
            Err(e) => eprintln!("Failed to connect to database: {}", e),
        }

        modal_state.set(ModalType::None);
    };

    let handle_delete = move |_| {
        let id = match *modal_state.read() {
            ModalType::Delete(id) => Some(id),
            _ => None,
        };

        if let Some(id) = id {
            match establish_connection() {
                Ok(conn) => {
                    if let Err(e) = delete_employee(&conn, id) {
                        eprintln!("Failed to delete employee: {}", e);
                    } else {
                        let mut emp_list = employees.read().clone();
                        emp_list.retain(|e| e.id != id);
                        employees.set(emp_list);
                    }
                }
                Err(e) => eprintln!("Failed to connect to database: {}", e),
            }
        }

        modal_state.set(ModalType::None);
    };

    let update_name = move |evt: FormEvent| {
        current_employee.write().name = evt.value();
    };

    let mut update_sex = move |sex: Sex| {
        current_employee.write().sex = sex;
    };

    let mut update_role = move |role: Role| {
        current_employee.write().role = role;
    };

    let mut update_required_days = move |days: u8| {
        current_employee.write().required_days = days;
    };

    let mut toggle_fixed_day = move |day: Weekday| {
        let mut days = current_employee.read().fixed_days.clone();
        if let Some(pos) = days.iter().position(|d| d == &day) {
            days.remove(pos);
        } else {
            days.push(day);
        }
        current_employee.write().fixed_days = days;
    };

    let toggle_nsp = move |_| {
        // Extract the current value first, then set the new value
        let current_value = current_employee.read().is_nsp;
        current_employee.write().is_nsp = !current_value;
    };

    let is_day_selected = move |day: &Weekday| current_employee.read().fixed_days.contains(day);

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: EMPLOYEES_CSS,
        }
        div { class: "employees-container",
            h1 { "Employees" }
            div { class: "header-actions",
                SearchBar {
                    placeholder: "Search employees...".to_string(),
                    on_search: handle_search
                }
                button {
                    class: "btn",
                    onclick: open_add_modal,
                    // img {
                    //     src: ADD_ICON,
                    //     width: "20",
                    //     height: "20",
                    // }
                    "Add Employee"
                }
            }
            div { class: "employee-cards",
                for employee in filtered_employees() {
                    div {
                        key: "{employee.id}",
                        class: "employee-card",
                        onclick: move |_| open_view_modal(employee.id),
                        div { class: "card-actions",
                            button {
                                class: "card-action-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    open_edit_modal(employee.id);
                                },
                                img {
                                    src: EDIT_ICON,
                                    width: "25",
                                    height: "25",
                                }
                            }
                            button {
                                class: "card-action-btn card-action-btn-delete",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    open_delete_modal(employee.id);
                                },
                                img {
                                    src: DELETE_ICON,
                                    width: "25",
                                    height: "25",
                                }
                            }
                        }
                        h3 { "{employee.name}" }
                        p { "{employee.role.to_string()}" }
                    }
                }
            }

            match *modal_state.read() {
                ModalType::Add | ModalType::Edit(_) => rsx! {
                    div { class: "modal-overlay",
                        div { class: "modal",
                            div { class: "modal-header",
                                h2 {
                                    if matches!(*modal_state.read(), ModalType::Add) {
                                        "Add New Employee"
                                    } else {
                                        "Edit Employee"
                                    }
                                }
                                button {
                                    class: "modal-close",
                                    onclick: close_modal,
                                    img {
                                        src: X_CLOSE_ICON,
                                        width: "40",
                                        height: "40",
                                    }
                                }
                            }
                            div { class: "modal-body",
                                div { class: "form-group",
                                    label { r#for: "name", "Name" }
                                    input {
                                        id: "name",
                                        class: "form-control",
                                        r#type: "text",
                                        value: "{current_employee.read().name}",
                                        oninput: update_name,
                                        placeholder: "Enter employee name"
                                    }
                                }
                                div { class: "form-group",
                                    label { "Sex" }
                                    div { class: "radio-group",
                                        for (id, val, label_text) in [("male", Sex::Male, "Male"), ("female", Sex::Female, "Female")] {
                                            div { class: "radio-option",
                                                input {
                                                    r#type: "radio",
                                                    id: "{id}",
                                                    name: "sex",
                                                    checked: current_employee.read().sex == val,
                                                    onclick: move |_| update_sex(val.clone())
                                                }
                                                label { r#for: "{id}", "{label_text}" }
                                            }
                                        }
                                    }
                                }
                                // div { class: "form-group",
                                //     label { "Role" }
                                //     div { class: "radio-group",
                                //         for (id, val, label_text) in [("hr", Role::HR, "HR"), ("ai-engineer", Role::AIEngineer, "Gen-AI"), ("full-stack", Role::FullStack, "Full-Stack")] {
                                //             div { class: "radio-option",
                                //                 input {
                                //                     r#type: "radio",
                                //                     id: "{id}",
                                //                     name: "role",
                                //                     checked: current_employee.read().role == val,
                                //                     onclick: move |_| update_role(val.clone())
                                //                 }
                                //                 label { r#for: "{id}", "{label_text}" }
                                //             }
                                //         }
                                //     }
                                // }
                                div { class: "form-group",
                                    label { "Role" }
                                    select {
                                        id: "role",
                                        class: "form-control role-select",
                                        onchange: move |event| {
                                            let value = event.value();
                                            let role = match value.as_str() {
                                                "hr" => Role::HR,
                                                "ai-llm-engineer" => Role::AiLlmEngineer,
                                                "social-media-marketing" => Role::SocialMediaMarketing,
                                                // "marketing-manager" => Role::MarketingManager,
                                                "it-support" => Role::ITSupport,
                                                "ml-engineer" => Role::MLEngineer,
                                                "data-scientist" => Role::DataScientist,
                                                "data-analyst" => Role::DataAnalyst,
                                                "full-stack-engineer" => Role::FullStackEngineer,
                                                "backend-engineer" => Role::BackendEngineer,
                                                "frontend-engineer" => Role::FrontendEngineer,
                                                "blockchain-engineer" => Role::BlockchainEngineer,
                                                "qa-engineer" => Role::QaEngineer,
                                                "project-manager" => Role::ProjectManager,
                                                "ui-ux-designer" => Role::UiUxDesigner,
                                                "mobile-engineer" => Role::MobileEngineer,
                                                "dev-ops-engineer" => Role::DevOpsEngineer,
                                                "operations-manager" => Role::OperationsManager,
                                                _ => Role::HR, // Default role
                                            };
                                            update_role(role);
                                        },
                                        option { value: "hr", selected: current_employee.read().role == Role::HR, "HR" }
                                        option { value: "ai-llm-engineer", selected: current_employee.read().role == Role::AiLlmEngineer, "AI LLM Engineer" }
                                        option { value: "social-media-marketing", selected: current_employee.read().role == Role::SocialMediaMarketing, "Social Media Marketing" }
                                        // option { value: "marketing-manager", selected: current_employee.read().role == Role::MarketingManager, "Marketing Manager" }
                                        option { value: "it-support", selected: current_employee.read().role == Role::ITSupport, "IT Support" }
                                        option { value: "ml-engineer", selected: current_employee.read().role == Role::MLEngineer, "ML Engineer" }
                                        option { value: "data-scientist", selected: current_employee.read().role == Role::DataScientist, "Data Scientist" }
                                        option { value: "data-analyst", selected: current_employee.read().role == Role::DataAnalyst, "Data Analyst" }
                                        option { value: "full-stack-engineer", selected: current_employee.read().role == Role::FullStackEngineer, "Full Stack Engineer" }
                                        option { value: "backend-engineer", selected: current_employee.read().role == Role::BackendEngineer, "Backend Engineer" }
                                        option { value: "frontend-engineer", selected: current_employee.read().role == Role::FrontendEngineer, "Frontend Engineer" }
                                        option { value: "blockchain-engineer", selected: current_employee.read().role == Role::BlockchainEngineer, "Blockchain Engineer" }
                                        option { value: "qa-engineer", selected: current_employee.read().role == Role::QaEngineer, "QA Engineer" }
                                        option { value: "project-manager", selected: current_employee.read().role == Role::ProjectManager, "Project Manager" }
                                        option { value: "ui-ux-designer", selected: current_employee.read().role == Role::UiUxDesigner, "UI/UX Designer" }
                                        option { value: "mobile-engineer", selected: current_employee.read().role == Role::MobileEngineer, "Mobile Engineer" }
                                        option { value: "dev-ops-engineer", selected: current_employee.read().role == Role::DevOpsEngineer, "DevOps Engineer" }
                                        option { value: "operations-manager", selected: current_employee.read().role == Role::OperationsManager, "Operations Manager" }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Required Days" }
                                    div { class: "radio-group",
                                        for (id, val) in [("days-1", 1), ("days-2", 2), ("days-3", 3), ("days-5", 5)] {
                                            div { class: "radio-option",
                                                input {
                                                    r#type: "radio",
                                                    id: "{id}",
                                                    name: "required-days",
                                                    checked: current_employee.read().required_days == val,
                                                    onclick: move |_| update_required_days(val)
                                                }
                                                label { r#for: "{id}", "{val}" }
                                            }
                                        }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Fixed Days" }
                                    div { class: "checkbox-group",
                                        for (id, day) in [
                                            ("monday", Weekday::Monday),
                                            ("tuesday", Weekday::Tuesday),
                                            ("wednesday", Weekday::Wednesday),
                                            ("thursday", Weekday::Thursday),
                                            ("friday", Weekday::Friday)
                                        ] {
                                            div { class: "checkbox-option",
                                                input {
                                                    r#type: "checkbox",
                                                    id: "{id}",
                                                    checked: is_day_selected(&day),
                                                    onclick: move |_| toggle_fixed_day(day.clone())
                                                }
                                                label { r#for: "{id}", "{day.to_string()}" }
                                            }
                                        }
                                    }
                                }
                                // div { class: "form-group",
                                //     div { class: "checkbox-option",
                                //         input {
                                //             r#type: "checkbox",
                                //             id: "is-nsp",
                                //             checked: current_employee.read().is_nsp,
                                //             onclick: toggle_nsp
                                //         }
                                //         label { r#for: "is-nsp", "Is NSP ?" }
                                //     }
                                // }
                                div { class: "form-group",
                                    label { r#for: "is-nsp", "Is NSP ?" }
                                    select {
                                        id: "is-nsp",
                                        class: "form-control nsp-select",
                                        onchange: toggle_nsp,
                                        option { value: "true", selected: current_employee.read().is_nsp, "Yes" }
                                        option { value: "false", selected: !current_employee.read().is_nsp, "No" }
                                    }
                                }
                            }
                            div { class: "modal-footer",
                                button {
                                    class: "btn btn-secondary",
                                    onclick: close_modal,
                                    "Cancel"
                                }
                                button {
                                    class: "btn btn-primary",
                                    onclick: handle_save,
                                    "Save"
                                }
                            }
                        }
                    }
                },
                ModalType::Delete(_) => rsx! {
                    div { class: "modal-overlay",
                        div { class: "modal modal-confirm",
                            div { class: "modal-header",
                                h2 { "Confirm Delete" }
                                button {
                                    class: "modal-close",
                                    onclick: close_modal,
                                    img {
                                        src: X_CLOSE_ICON,
                                        width: "40",
                                        height: "40",
                                    }
                                }
                            }
                            div { class: "modal-body",
                                p { "Are you sure you want to delete this employee?" }
                                div { class: "employee-detail",
                                    div { class: "detail-row",
                                        strong { "Name: " }
                                        span { "{current_employee.read().name}" }
                                    }
                                    div { class: "detail-row",
                                        strong { "Role: " }
                                        span { "{current_employee.read().role.to_string()}" }
                                    }
                                }
                            }
                            div { class: "modal-footer",
                                button {
                                    class: "btn btn-secondary",
                                    onclick: close_modal,
                                    "Cancel"
                                }
                                button {
                                    class: "btn btn-danger",
                                    onclick: handle_delete,
                                    "Delete"
                                }
                            }
                        }
                    }
                },
                ModalType::View(_) => rsx! {
                    div { class: "modal-overlay",
                        div { class: "modal",
                            div { class: "modal-header",
                                h2 { "Employee Details" }
                                button {
                                    class: "modal-close",
                                    onclick: close_modal,
                                    img {
                                        src: X_CLOSE_ICON,
                                        width: "40",
                                        height: "40",
                                    }
                                }
                            }
                            div { class: "modal-body",
                                div { class: "employee-detail",
                                    for (label, value) in [
                                        ("Name:", current_employee.read().name.clone()),
                                        ("Sex:", current_employee.read().sex.to_string()),
                                        ("Role:", current_employee.read().role.to_string()),
                                        ("Required Days:", current_employee.read().required_days.to_string()),
                                        ("Fixed Days:", current_employee.read().fixed_days.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ")),
                                        ("Is NSP:", (if current_employee.read().is_nsp { "Yes" } else { "No" }).to_string()),
                                    ] {
                                        div { class: "detail-row",
                                            span { class: "detail-label", "{label}" }
                                            span { class: "detail-value", "{value}" }
                                        }
                                    }
                                }
                            }
                            div { class: "modal-footer",
                                button {
                                    class: "btn btn-secondary",
                                    onclick: close_modal,
                                    "Close"
                                }
                            }
                        }
                    }
                },
                ModalType::None => rsx! {}
            }
        }
    }
}
