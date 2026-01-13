//! SUPP (Supplemental Qualifiers) tab view.
//!
//! The SUPP tab allows configuration of supplemental qualifier domains
//! for columns that don't map to standard SDTM variables.
//!
//! Features:
//! - Master list of unmapped source columns
//! - Detail panel for SUPP configuration (QNAM, QLABEL, QORIG, QEVAL)
//! - Inline editing with auto-uppercase QNAM
//! - QNAM uniqueness validation
//! - Sample data preview from source column

use iced::widget::{
    Space, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Border, Element, Length, Theme};
use iced_fonts::lucide;
use polars::prelude::AnyValue;

use crate::message::domain_editor::SuppMessage;
use crate::message::{DomainEditorMessage, Message};
use crate::state::{
    AppState, Domain, SuppAction, SuppColumnConfig, SuppFilterMode, SuppOrigin, SuppUiState,
    ViewState,
};
use crate::theme::{
    BORDER_RADIUS_SM, ERROR, GRAY_100, GRAY_200, GRAY_300, GRAY_400, GRAY_500, GRAY_600, GRAY_700,
    GRAY_800, GRAY_900, PRIMARY_100, PRIMARY_500, SPACING_LG, SPACING_MD, SPACING_SM, SPACING_XS,
    SUCCESS, WHITE,
};

// =============================================================================
// CONSTANTS
// =============================================================================

const MASTER_WIDTH: f32 = 300.0;
const QNAM_MAX_LEN: usize = 8;
const QLABEL_MAX_LEN: usize = 40;

// =============================================================================
// MAIN SUPP TAB VIEW
// =============================================================================

/// Render the SUPP configuration tab content.
pub fn view_supp_tab<'a>(state: &'a AppState, domain_code: &'a str) -> Element<'a, Message> {
    let domain = match state.domain(domain_code) {
        Some(d) => d,
        None => {
            return container(text("Domain not found").size(14).color(GRAY_500))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Shrink)
                .center_y(Length::Shrink)
                .into();
        }
    };

    // Get UI state
    let supp_ui = match &state.view {
        ViewState::DomainEditor { supp_ui, .. } => supp_ui,
        _ => return text("Invalid view state").into(),
    };

    // Get unmapped columns
    let unmapped_columns = domain.unmapped_columns();

    // If no unmapped columns, show success state
    if unmapped_columns.is_empty() {
        return view_all_mapped_state(domain_code);
    }

    // Build master list inline to avoid lifetime issues
    let master = build_master_list(&unmapped_columns, domain, supp_ui, domain_code);

    // Build detail panel
    let detail = build_detail_panel(domain, supp_ui, domain_code);

    // Vertical divider
    let divider = container(Space::new().width(1))
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(|_: &Theme| container::Style {
            background: Some(GRAY_200.into()),
            ..Default::default()
        });

    row![
        // Master list
        container(master)
            .width(Length::Fixed(MASTER_WIDTH))
            .height(Length::Fill),
        // Divider
        divider,
        // Detail panel
        container(detail).width(Length::Fill).height(Length::Fill),
    ]
    .height(Length::Fill)
    .into()
}

// =============================================================================
// MASTER LIST (builds elements that own their data)
// =============================================================================

/// Build master list showing unmapped columns.
fn build_master_list(
    columns: &[String],
    domain: &Domain,
    ui: &SuppUiState,
    domain_code: &str,
) -> Element<'static, Message> {
    // Filter columns based on search and filter mode
    let filtered: Vec<String> = columns
        .iter()
        .filter(|col| {
            // Search filter
            if !ui.search_filter.is_empty()
                && !col.to_lowercase().contains(&ui.search_filter.to_lowercase())
            {
                return false;
            }

            // Action filter
            let config = domain.supp_config.get(*col);
            match ui.filter_mode {
                SuppFilterMode::All => true,
                SuppFilterMode::Pending => {
                    config.map_or(true, |c| c.action == SuppAction::Pending)
                }
                SuppFilterMode::Included => {
                    config.map_or(false, |c| c.action == SuppAction::Include)
                }
                SuppFilterMode::Skipped => config.map_or(false, |c| c.action == SuppAction::Skip),
            }
        })
        .cloned()  // Clone to own the data
        .collect();

    // Header
    let header = build_master_header(domain_code, filtered.len());

    // Search box - clone the filter for the closure
    let search_filter = ui.search_filter.clone();
    let search = text_input("Search columns...", &search_filter)
        .on_input(|s| {
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::SearchChanged(s)))
        })
        .padding([8.0, 12.0])
        .size(13);

    // Filter buttons
    let filters = build_filter_buttons(ui.filter_mode);

    // Column list
    let column_list: Element<'static, Message> = if filtered.is_empty() {
        container(
            column![
                lucide::search_x().size(32).color(GRAY_400),
                Space::new().height(SPACING_SM),
                text("No columns match filter").size(13).color(GRAY_500),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fixed(150.0))
        .center_x(Length::Shrink)
        .center_y(Length::Shrink)
        .into()
    } else {
        // Build items with owned data
        let selected_col = ui.selected_column.clone();
        let items: Vec<Element<'static, Message>> = filtered
            .into_iter()
            .map(|col_name| {
                let config = domain.supp_config.get(&col_name);
                let action = config.map_or(SuppAction::Pending, |c| c.action);
                let is_selected = selected_col.as_deref() == Some(col_name.as_str());
                build_column_item(col_name, action, is_selected)
            })
            .collect();

        scrollable(column(items).spacing(2).width(Length::Fill))
            .height(Length::Fill)
            .into()
    };

    column![
        header,
        Space::new().height(SPACING_SM),
        search,
        Space::new().height(SPACING_SM),
        filters,
        Space::new().height(SPACING_SM),
        column_list,
    ]
    .padding(SPACING_MD)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Build master list header.
fn build_master_header(domain_code: &str, count: usize) -> Element<'static, Message> {
    let title = format!("SUPP{}", domain_code);
    let subtitle = format!("{} unmapped columns", count);

    column![
        text(title).size(16).color(GRAY_900).font(iced::Font {
            weight: iced::font::Weight::Semibold,
            ..Default::default()
        }),
        Space::new().height(2.0),
        text(subtitle).size(12).color(GRAY_500),
    ]
    .into()
}

/// Build filter buttons.
fn build_filter_buttons(current: SuppFilterMode) -> Element<'static, Message> {
    let filters = [
        (SuppFilterMode::All, "All"),
        (SuppFilterMode::Pending, "Pending"),
        (SuppFilterMode::Included, "SUPP"),
        (SuppFilterMode::Skipped, "Skip"),
    ];

    let buttons: Vec<Element<'static, Message>> = filters
        .iter()
        .map(|(mode, label)| {
            let is_selected = current == *mode;
            let mode_val = *mode;
            let label_str = *label;

            button(
                text(label_str)
                    .size(11)
                    .color(if is_selected { PRIMARY_500 } else { GRAY_600 }),
            )
            .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
                SuppMessage::FilterModeChanged(mode_val),
            )))
            .padding([4.0, 8.0])
            .style(move |_: &Theme, _status| {
                if is_selected {
                    iced::widget::button::Style {
                        background: Some(PRIMARY_100.into()),
                        text_color: PRIMARY_500,
                        border: Border {
                            color: PRIMARY_500,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                } else {
                    iced::widget::button::Style {
                        background: Some(WHITE.into()),
                        text_color: GRAY_600,
                        border: Border {
                            color: GRAY_300,
                            width: 1.0,
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                }
            })
            .into()
        })
        .collect();

    row(buttons).spacing(4.0).into()
}

/// Build single column item in the master list.
fn build_column_item(
    col_name: String,
    action: SuppAction,
    is_selected: bool,
) -> Element<'static, Message> {
    // Status indicator
    let status_icon: Element<'static, Message> = match action {
        SuppAction::Pending => lucide::circle().size(10).color(GRAY_400).into(),
        SuppAction::Include => lucide::circle_check().size(10).color(SUCCESS).into(),
        SuppAction::Skip => lucide::circle_x().size(10).color(GRAY_400).into(),
    };

    let bg_color = if is_selected { PRIMARY_100 } else { WHITE };
    let text_color = if is_selected { PRIMARY_500 } else { GRAY_800 };
    let display_name = col_name.clone();

    button(
        row![
            status_icon,
            Space::new().width(SPACING_SM),
            text(display_name).size(13).color(text_color),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
        SuppMessage::ColumnSelected(col_name),
    )))
    .padding([8.0, 12.0])
    .width(Length::Fill)
    .style(move |_: &Theme, _status| iced::widget::button::Style {
        background: Some(bg_color.into()),
        text_color,
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

// =============================================================================
// DETAIL PANEL
// =============================================================================

/// Build detail panel for configuring a selected column.
fn build_detail_panel(
    domain: &Domain,
    ui: &SuppUiState,
    domain_code: &str,
) -> Element<'static, Message> {
    match &ui.selected_column {
        Some(col) => build_column_detail(domain, col.clone(), domain_code.to_string()),
        None => build_no_selection_state(),
    }
}

/// Build no column selected state.
fn build_no_selection_state() -> Element<'static, Message> {
    container(
        column![
            lucide::mouse_pointer_click().size(48).color(GRAY_400),
            Space::new().height(SPACING_LG),
            text("Select a Column").size(18).color(GRAY_700),
            Space::new().height(SPACING_SM),
            text("Click on a column in the list to configure its SUPP settings")
                .size(13)
                .color(GRAY_500),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}

/// Build detail view for a selected column.
fn build_column_detail(
    domain: &Domain,
    col_name: String,
    domain_code: String,
) -> Element<'static, Message> {
    // Get or create config for this column
    let config = domain
        .supp_config
        .get(&col_name)
        .cloned()
        .unwrap_or_else(|| SuppColumnConfig::from_column(&col_name));

    // Check for QNAM uniqueness
    let qnam_conflict = check_qnam_conflict(domain, &col_name, &config.qnam);

    // Header
    let header = build_detail_header(&col_name, &domain_code);

    // Sample data preview
    let sample_data = build_sample_data(domain, &col_name);

    // SUPP fields
    let fields = build_supp_fields(col_name.clone(), config.clone(), qnam_conflict);

    // Action selector
    let action_selector = build_action_selector(col_name, config.action);

    scrollable(
        column![
            header,
            Space::new().height(SPACING_LG),
            sample_data,
            Space::new().height(SPACING_LG),
            fields,
            Space::new().height(SPACING_LG),
            action_selector,
        ]
        .padding(SPACING_LG)
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .into()
}

/// Build detail header.
fn build_detail_header(col_name: &str, domain_code: &str) -> Element<'static, Message> {
    let col_display = col_name.to_string();
    let target = format!("SUPP{}", domain_code);

    column![
        text("Configure SUPP Variable")
            .size(18)
            .color(GRAY_900)
            .font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            }),
        Space::new().height(4.0),
        row![
            text("Source Column:").size(13).color(GRAY_500),
            Space::new().width(SPACING_XS),
            text(col_display).size(13).color(GRAY_800).font(iced::Font {
                weight: iced::font::Weight::Semibold,
                ..Default::default()
            }),
        ],
        Space::new().height(2.0),
        row![
            text("Target:").size(13).color(GRAY_500),
            Space::new().width(SPACING_XS),
            text(target).size(13).color(GRAY_600),
        ],
    ]
    .into()
}

/// Build sample data from source column.
fn build_sample_data(domain: &Domain, col_name: &str) -> Element<'static, Message> {
    let samples = get_sample_values(domain, col_name, 5);

    let sample_chips: Vec<Element<'static, Message>> = samples
        .into_iter()
        .map(|s| {
            container(text(s).size(11).color(GRAY_700))
                .padding([4.0, 8.0])
                .style(|_: &Theme| container::Style {
                    background: Some(GRAY_100.into()),
                    border: Border {
                        radius: BORDER_RADIUS_SM.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        })
        .collect();

    let sample_content: Element<'static, Message> = if sample_chips.is_empty() {
        text("No data available").size(12).color(GRAY_400).into()
    } else {
        row(sample_chips).spacing(SPACING_XS).wrap().into()
    };

    container(
        column![
            row![
                lucide::database().size(14).color(GRAY_500),
                Space::new().width(SPACING_SM),
                text("Sample Values").size(13).color(GRAY_600),
            ]
            .align_y(Alignment::Center),
            Space::new().height(SPACING_SM),
            sample_content,
        ]
        .width(Length::Fill),
    )
    .padding(SPACING_MD)
    .style(|_: &Theme| container::Style {
        background: Some(GRAY_100.into()),
        border: Border {
            radius: BORDER_RADIUS_SM.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// Get sample values from a column.
fn get_sample_values(domain: &Domain, col_name: &str, max: usize) -> Vec<String> {
    let mut samples = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Ok(col) = domain.source.data.column(col_name) {
        for i in 0..col.len().min(100) {
            if let Ok(val) = col.get(i) {
                let s = format_value(&val);
                if !s.is_empty() && seen.insert(s.clone()) {
                    samples.push(s);
                    if samples.len() >= max {
                        break;
                    }
                }
            }
        }
    }

    samples
}

/// Format a Polars value for display.
fn format_value(value: &AnyValue) -> String {
    match value {
        AnyValue::Null => String::new(),
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        _ => format!("{}", value),
    }
}

/// Build SUPP configuration fields.
fn build_supp_fields(
    col_name: String,
    config: SuppColumnConfig,
    qnam_error: Option<String>,
) -> Element<'static, Message> {
    // QNAM field
    let qnam_field = build_text_field(
        "QNAM",
        "Qualifier Variable Name (max 8 chars)",
        config.qnam.clone(),
        QNAM_MAX_LEN,
        qnam_error,
        {
            let col = col_name.clone();
            move |v| {
                Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QnamChanged {
                    column: col.clone(),
                    value: v.to_uppercase(), // Auto-uppercase
                }))
            }
        },
    );

    // QLABEL field
    let qlabel_field = build_text_field(
        "QLABEL",
        "Qualifier Variable Label (max 40 chars)",
        config.qlabel.clone(),
        QLABEL_MAX_LEN,
        None,
        {
            let col = col_name.clone();
            move |v| {
                Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QlabelChanged {
                    column: col.clone(),
                    value: v,
                }))
            }
        },
    );

    // QORIG picker
    let qorig_field = build_origin_picker(col_name.clone(), config.qorig);

    // QEVAL field
    let qeval_field = build_text_field(
        "QEVAL (Optional)",
        "Evaluator (e.g., INVESTIGATOR)",
        config.qeval.unwrap_or_default(),
        40,
        None,
        {
            let col = col_name;
            move |v| {
                Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QevalChanged {
                    column: col.clone(),
                    value: v,
                }))
            }
        },
    );

    column![qnam_field, qlabel_field, qorig_field, qeval_field,]
        .spacing(SPACING_MD)
        .into()
}

/// Build text input field with label and validation.
fn build_text_field<F>(
    label: &'static str,
    placeholder: &'static str,
    value: String,
    max_len: usize,
    error: Option<String>,
    on_change: F,
) -> Element<'static, Message>
where
    F: 'static + Fn(String) -> Message,
{
    let char_count = value.len();
    let is_over = char_count > max_len;
    let has_error = error.is_some() || is_over;

    let error_msg: Element<'static, Message> = if let Some(err) = error {
        row![
            lucide::circle_alert().size(12).color(ERROR),
            Space::new().width(4.0),
            text(err).size(11).color(ERROR),
        ]
        .into()
    } else {
        Space::new().height(0.0).into()
    };

    let count_display = format!("{}/{}", char_count, max_len);

    column![
        row![
            text(label).size(12).color(GRAY_600),
            Space::new().width(Length::Fill),
            text(count_display)
                .size(11)
                .color(if is_over { ERROR } else { GRAY_400 }),
        ],
        Space::new().height(4.0),
        text_input(placeholder, &value)
            .on_input(on_change)
            .padding([10.0, 12.0])
            .size(14)
            .style(move |_: &Theme, _status| {
                let border_color = if has_error { ERROR } else { GRAY_300 };
                iced::widget::text_input::Style {
                    background: WHITE.into(),
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius: BORDER_RADIUS_SM.into(),
                    },
                    icon: GRAY_500,
                    placeholder: GRAY_400,
                    value: GRAY_900,
                    selection: PRIMARY_100,
                }
            }),
        error_msg,
    ]
    .into()
}

/// Build origin picker dropdown.
fn build_origin_picker(col_name: String, current: SuppOrigin) -> Element<'static, Message> {
    column![
        text("QORIG").size(12).color(GRAY_600),
        Space::new().height(4.0),
        pick_list(&SuppOrigin::ALL[..], Some(current), move |origin| {
            Message::DomainEditor(DomainEditorMessage::Supp(SuppMessage::QorigChanged {
                column: col_name.clone(),
                value: origin,
            }))
        })
        .text_size(14)
        .padding([10.0, 12.0])
        .width(Length::Fill),
    ]
    .into()
}

/// Build action selector (Pending/Include/Skip).
fn build_action_selector(col_name: String, current: SuppAction) -> Element<'static, Message> {
    let actions = [
        (SuppAction::Pending, "Pending", GRAY_500, "Not yet decided"),
        (
            SuppAction::Include,
            "Add to SUPP",
            SUCCESS,
            "Include in SUPP domain output",
        ),
        (
            SuppAction::Skip,
            "Skip",
            GRAY_400,
            "Don't include in output",
        ),
    ];

    let buttons: Vec<Element<'static, Message>> = actions
        .iter()
        .map(|(action, label, color, desc)| {
            let is_selected = current == *action;
            let action_val = *action;
            let col_clone = col_name.clone();
            let icon_color = *color;

            let icon: Element<'static, Message> = if is_selected {
                lucide::circle_check().size(16).color(icon_color).into()
            } else {
                lucide::circle().size(16).color(GRAY_300).into()
            };

            container(
                button(column![
                    row![
                        icon,
                        Space::new().width(SPACING_SM),
                        text(*label)
                            .size(14)
                            .color(if is_selected { GRAY_900 } else { GRAY_600 }),
                    ]
                    .align_y(Alignment::Center),
                    Space::new().height(2.0),
                    text(*desc).size(11).color(GRAY_500),
                ])
                .on_press(Message::DomainEditor(DomainEditorMessage::Supp(
                    SuppMessage::ActionChanged {
                        column: col_clone,
                        action: action_val,
                    },
                )))
                .padding(SPACING_MD)
                .width(Length::Fill)
                .style(move |_: &Theme, _status| {
                    let bg = if is_selected { PRIMARY_100 } else { WHITE };
                    let border_color = if is_selected { PRIMARY_500 } else { GRAY_200 };
                    iced::widget::button::Style {
                        background: Some(bg.into()),
                        border: Border {
                            color: border_color,
                            width: if is_selected { 2.0 } else { 1.0 },
                            radius: BORDER_RADIUS_SM.into(),
                        },
                        ..Default::default()
                    }
                }),
            )
            .width(Length::FillPortion(1))
            .into()
        })
        .collect();

    column![
        text("Action").size(12).color(GRAY_600),
        Space::new().height(SPACING_SM),
        row(buttons).spacing(SPACING_SM),
    ]
    .into()
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Check if QNAM conflicts with another column.
fn check_qnam_conflict(domain: &Domain, current_col: &str, qnam: &str) -> Option<String> {
    if qnam.is_empty() {
        return None;
    }

    for (col, config) in &domain.supp_config {
        if col != current_col && config.qnam.eq_ignore_ascii_case(qnam) {
            return Some(format!("QNAM '{}' already used by column '{}'", qnam, col));
        }
    }

    None
}

// =============================================================================
// STATES
// =============================================================================

/// All columns mapped state.
fn view_all_mapped_state(domain_code: &str) -> Element<'static, Message> {
    let msg1 = format!(
        "All source columns are mapped to {} variables.",
        domain_code
    );

    container(
        column![
            lucide::circle_check().size(48).color(SUCCESS),
            Space::new().height(SPACING_LG),
            text("All Columns Mapped").size(18).color(GRAY_700),
            Space::new().height(SPACING_SM),
            text(msg1).size(13).color(GRAY_500),
            Space::new().height(4.0),
            text("No SUPP configuration needed.")
                .size(13)
                .color(GRAY_500),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Shrink)
    .center_y(Length::Shrink)
    .into()
}

// =============================================================================
// DISPLAY IMPLEMENTATIONS
// =============================================================================

impl std::fmt::Display for SuppOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}
