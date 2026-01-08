use crate::app::Message;
use crate::state::AppState;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Theme};

pub struct EmployeesView;

impl EmployeesView {
    pub fn view(state: &AppState) -> Element<Message, Theme> {
        let employees = state.get_filtered_employees();
        let is_editing = state.is_editing;

        let mut content = column![
            text("Employees").size(24),
            row![
                button("Add Employee").on_press(Message::AddEmployee),
                button("Import").on_press(Message::ImportEmployees),
                button("Export").on_press(Message::ExportEmployees),
                button("Save").on_press(Message::SaveEmployees),
            ]
            .spacing(10),
        ]
        .spacing(20);

        if employees.is_empty() {
            content = content.push(text(
                "No employees found. Click 'Add Employee' to get started.",
            ));
        } else {
            // Table header
            let header = row![
                text("ID").width(50),
                text("Name").width(200),
                text("Sex").width(80),
                text("Role").width(150),
                text("Required Days").width(120),
                text("Actions").width(150),
            ]
            .spacing(10)
            .padding(10);

            content = content.push(header);

            let mut employee_list = column![].spacing(5);

            // Employee rows
            for emp in employees.iter() {
                let emp_id = emp.id;
                let emp_name = emp.name.clone();
                let emp_sex = emp.sex.to_string();
                let emp_role = emp.role.map(|r| r.to_string()).unwrap_or_default();
                let emp_days = emp.required_days;

                let name_widget: Element<Message, Theme> = if is_editing {
                    text_input("Name", &emp_name)
                        .on_input(move |name| Message::EmployeeNameChanged(emp_id, name))
                        .into()
                } else {
                    text(emp_name).into()
                };

                let days_widget: Element<Message, Theme> = if is_editing {
                    text_input("Days", &emp_days.to_string())
                        .on_input(move |days| {
                            if let Ok(d) = days.parse::<u8>() {
                                Message::EmployeeRequiredDaysChanged(emp_id, d)
                            } else {
                                Message::ClearStatus
                            }
                        })
                        .into()
                } else {
                    text(emp_days).into()
                };

                let row_widget = row![
                    text(emp_id).width(50),
                    container(name_widget).width(200),
                    text(emp_sex).width(80),
                    text(emp_role).width(150),
                    container(days_widget).width(120),
                    button("Delete")
                        .on_press(Message::DeleteEmployee(emp_id))
                        .width(150),
                ]
                .spacing(10)
                .padding(5);

                employee_list = employee_list.push(row_widget);
            }

            content = content.push(scrollable(employee_list).height(Length::Fill));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
