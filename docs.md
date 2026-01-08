# Office Scheduler - GPUI Application

A modular office scheduling application built with GPUI, featuring intelligent schedule generation with configurable constraints.

## Architecture

The application is structured as a workspace with 5 independent crates:

- **core**: Data models and shared types
- **scheduler**: Schedule generation logic with constraints
- **storage**: SurrealDB embedded database layer
- **integrations**: Google Sheets and Discord integration scaffolding
- **ui**: GPUI-based user interface

## Features

### Schedule Generation
- Random balanced scheduling across weekdays
- Fixed day assignments
- Configurable sex/role distribution
- Mentor-mentee day overlap constraints
- Historical schedule consideration to avoid repetition

### Employee Management
- CRUD operations with undo/redo
- Search and filtering
- Import from JSON
- Mentor/mentee relationships

### Integrations
- Google Sheets import/export (scaffolding ready)
- Discord notifications (scaffolding ready)
- Configurable sharing permissions

### UI Features
- Three main tabs: Employees, Schedules, Settings
- Edit mode with save/revert
- Keyboard shortcuts (configurable)
- Search across all tabs
- Modal dialogs for confirmations

## Building

### Prerequisites

```bash
# macOS
brew install cmake openssl

# Ubuntu/Debian
sudo apt-get install cmake libssl-dev pkg-config

# Windows
# Install Visual Studio Build Tools
# Install CMake
```

### Development Build

```bash
# Clone and build
git clone <repo-url>
cd office-scheduler
cargo build
```

### Release Build

```bash
cargo build --release
```

### Cross-Compilation to Windows (from macOS)

```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Install mingw-w64
brew install mingw-w64

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

## Running

```bash
# Development
cargo run

# Release
cargo run --release

# Or run the binary directly
./target/release/office-scheduler
```

## Testing

Each crate has its own test suite:

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p core
cargo test -p scheduler
cargo test -p storage

# Run with output
cargo test -- --nocapture
```

## Configuration

The application stores data in platform-specific directories:

- **macOS**: `~/Library/Application Support/office-scheduler/`
- **Windows**: `%APPDATA%\office-scheduler\`
- **Linux**: `~/.config/office-scheduler/`

### Default Configuration

```json
{
  "schedule": {
    "consider_sex_distribution": false,
    "consider_role_distribution": false,
    "mentee_mentor_overlap": "None"
  },
  "integrations": {
    "google_enabled": false,
    "discord_enabled": false
  },
  "shortcuts": {
    "save": "cmd+s",
    "delete": "cmd+backspace",
    "undo": "cmd+z",
    "redo": "cmd+shift+z"
  }
}
```

## Usage Guide

### Adding Employees

1. Navigate to Employees tab
2. Click "Edit" button
3. Click "Add" to create new employee
4. Fill in details:
   - Name (required)
   - Sex
   - Role (optional)
   - Required days per week
   - Fixed days (if any)
   - Mentor/mentee status

### Generating Schedules

1. Navigate to Schedules tab
2. Select month and year
3. Click "Generate"
4. Review generated schedule
5. Click "Edit" to make manual adjustments
6. Click "Save" to persist

### Configuring Settings

1. Navigate to Settings tab
2. Toggle distribution considerations:
   - Sex distribution: Balances male/female across days
   - Role distribution: Balances roles across days
3. Set mentee overlap mode:
   - None: No constraint
   - At least 1 day: Mentee shares at least one day with mentor
   - At least 2 days: Mentee shares at least two days with mentor
4. Configure integrations (when implemented)

### Import/Export

**Import Employees (JSON):**
```json
[
  {
    "name": "Alice Smith",
    "sex": "Female",
    "role": "Full-stack Engineer",
    "required_days": 2,
    "fixed_days": ["Monday"],
    "is_nsp": false
  }
]
```

**Export Schedule:**
- Click "Export" on Schedules tab
- Choose format: Local file or Google Sheets
- For Google Sheets: Configure sharing permissions in Settings

## Keyboard Shortcuts

Default shortcuts (configurable in Settings):

- `Cmd/Ctrl + S`: Save
- `Cmd/Ctrl + Z`: Undo
- `Cmd/Ctrl + Shift + Z`: Redo
- `Cmd/Ctrl + Backspace`: Delete selected
- `Cmd/Ctrl + 1`: Switch to Employees tab
- `Cmd/Ctrl + 2`: Switch to Schedules tab
- `Cmd/Ctrl + 3`: Switch to Settings tab
- `Cmd/Ctrl + F`: Focus search
- `Cmd/Ctrl + W`: Close window
- `Cmd/Ctrl + M`: Minimize
- `Cmd/Ctrl + Ctrl + F`: Maximize (macOS)

## Development Guide

### Adding New Features

1. **Core models**: Add to `crates/core/src/`
2. **Scheduling logic**: Modify `crates/scheduler/src/generator.rs`
3. **Storage**: Update `crates/storage/src/database.rs`
4. **UI components**: Add to `crates/ui/src/components/`

### Testing New Components

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_feature() {
        // Test implementation
    }
}
```

### Implementing Integrations

The integration scaffolding is ready. To implement:

1. Add API client dependencies to `crates/integrations/Cargo.toml`
2. Implement authentication in `google.rs` or `discord.rs`
3. Add API calls to existing placeholder methods
4. Update UI to handle integration status

## Troubleshooting

### Database Issues

```bash
# Clear database (will delete all data)
rm -rf ~/Library/Application\ Support/office-scheduler/data.db
```

### Build Errors

```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update
```

### GPUI Issues

Ensure you're using the latest GPUI commit:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "main" }
```

## Contributing

1. Write tests for new features
2. Ensure all tests pass: `cargo test --workspace`
3. Format code: `cargo fmt`
4. Run clippy: `cargo clippy`

## License

[Your License Here]

## Acknowledgments

- Built with [GPUI](https://github.com/zed-industries/zed)
- Database: [SurrealDB](https://surrealdb.com/)
