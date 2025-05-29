# daysApp

A simple desktop application for assigning days to employees based on a specific criteria.


## Dependencies

The project uses the following main dependencies, defined in `Cargo.toml`:

- **anyhow**: For flexible error handling.
- **chrono**: For date and time management.
- **dioxus**: A React-like framework for building user interfaces.
- **dioxus-desktop**: Enables building desktop applications with Dioxus.
- **dirs**: Provides a way to determine platform-specific directories.
- **rand**: For random number generation.
- **rfd**: A cross-platform file dialog library.
- **rusqlite**: A lightweight SQLite library.
- **rust_xlsxwriter**: For writing Excel files.
- **serde**: For serialization and deserialization.
- **serde_json**: For handling JSON data.
- **tao**: A windowing toolkit.


## Key Features

- **Employee Management**: Add, edit, and delete employee records, including their roles, required office days, and fixed days.
- **Automated Scheduling**: Generate balanced office schedules based on employee requirements and preferences.
- **Schedule Saving/Loading**: Persist schedules to a local SQLite database for later retrieval and modification.
- **Flexible Date Selection**: Select a specific month and year for schedule generation.
- **Exporting Schedules**: Export schedules as `.xlsx` files for easy sharing and integration with other tools.
- **Customizable UI**: A user-friendly interface built with Dioxus.
- **Import**: You can import employees through a json file.
- **Settings**: Allows to clear employee and schedule data.


## System Components and Usage

-   **`src/main.rs`**:
    *   Serves as the entry point to the application.
    *   Initializes the Dioxus desktop application.
    *   Sets up the SQLite database connection and creates necessary tables (`employees`, `schedules`).
-   **`src/client/app.rs`**:
    *   Defines the root component of the Dioxus application.
    *   Sets up the router for navigation between different pages.
    *   Includes global CSS.
-   **`src/client/pages`**:
    *   Contains the main pages of the application:
        *   `EmployeesPage`: Manages employee records.
        *   `SchedulesPage`: Generates and displays office schedules.
        *   `SettingsPage`: Provides data management and import/export options.
-   **`src/client/components`**:
    *   Includes reusable UI components:
        *   `NavBar`: Provides navigation between the main pages.
        *   `SearchBar`: Implements search functionality.
        *   `ImportButton`: Handles importing employee data from a JSON file.
        *   `ShareButton`: Enables exporting schedules to an XLSX file.
-   **`src/server`**:
    *   Houses the application's server-side logic:
        *   `db.rs`: Manages database connections and operations.
        *   `scheduler.rs`: Contains the scheduling algorithm and logic for generating balanced schedules.
        *   `export.rs`: Handles exporting schedules to `.xlsx` format.
        *   `import.rs`: Manages employee import.
        *   `schema.rs`: Defines the data structures used in the application (e.g., `Employee`, `Sex`, `Role`, `Weekday`, `MonthlySchedule`).


## System Setup Guide

1.  **Install Rust**:

    *   If you don't have Rust installed, download and install it from [https://www.rust-lang.org/](https://www.rust-lang.org/).

2.  **Install Dioxus CLI**:

    *   The [Dioxus CLI](https://dioxuslabs.com/docs/0.6/reference/command_line_tool/) is the primary tool for building and serving Dioxus applications. Install it using:
        ```bash
        cargo install dioxus-cli
        ```

3.  **Clone the Repository**:

    ```bash
    git clone https://github.com/felixLandlord/daysApp.git
    cd daysApp
    ```

4.  **Install Dependencies**:

    *   Use Cargo to install the project's dependencies:

    ```bash
    cargo build --release
    ```

5.  **Serve the Application**:

    *   Use the Dioxus CLI to serve the application during development:

    ```bash
    dx serve --platform desktop
    ```

    *   This command compiles the application and starts a development server with hot-reloading.


### Bundling Your App

Run the following command in the root of your project to start bundling with the default platform for distribution:

```bash
dx build --platform desktop --release
```
