use crate::{delete_icon, style, Message};
use coldmaps::{
    filters::{
        Distance2DFilter, Distance3DFilter, Filter, KillerClassFilter, KillerElevationFilter, KillerTeamFilter, OrderedOperator, Property, PropertyFilter, PropertyOperator,
        RoundFilter, VictimClassFilter, VictimElevationFilter, VictimTeamFilter,
    },
    heatmap_analyser::Team,
};
use iced::{button, pick_list, scrollable, text_input, Button, Column, Container, Element, Font, HorizontalAlignment, Length, PickList, Row, Scrollable, Text, TextInput};
use std::fmt::Display;
use style::ActiveButtonHighlight;

const CLASS_ICONS: Font = Font::External {
    name: "ClassIcons",
    bytes: include_bytes!("../fonts/tf2-classicons.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(CLASS_ICONS)
        .color([0.0, 0.0, 0.0])
        .horizontal_alignment(HorizontalAlignment::Center)
        .size(20)
}

const CLASS_ICONS_CHARS: [char; 10] = [
    '?',    // Other
    '🐇', // Scout
    '🎷', // Sniper
    '💥', // Soldier
    '💣', // Demoman
    '💗', // Medic
    '💪', // Heavy
    '🔥', // Pyro
    '📦', // Spy
    '🔧', // Engineer
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    KillerTeamFilter,
    VictimTeamFilter,
    KillerClassFilter,
    VictimClassFilter,
    KillerElevationFilter,
    VictimElevationFilter,
    Distance2DFilter,
    Distance3DFilter,
    RoundFilter,
    PropertyFilter,
}

impl FilterType {
    const ALL: [FilterType; 10] = [
        FilterType::KillerTeamFilter,
        FilterType::VictimTeamFilter,
        FilterType::KillerClassFilter,
        FilterType::VictimClassFilter,
        FilterType::KillerElevationFilter,
        FilterType::VictimElevationFilter,
        FilterType::Distance2DFilter,
        FilterType::Distance3DFilter,
        FilterType::RoundFilter,
        FilterType::PropertyFilter,
    ];
}

impl Default for FilterType {
    fn default() -> Self {
        FilterType::KillerTeamFilter
    }
}

impl Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterType::KillerTeamFilter => write!(f, "Killer's team"),
            FilterType::VictimTeamFilter => write!(f, "Victim's team"),
            FilterType::KillerClassFilter => write!(f, "Killer's class"),
            FilterType::VictimClassFilter => write!(f, "Victim's class"),
            FilterType::KillerElevationFilter => write!(f, "Killer's elevation"),
            FilterType::VictimElevationFilter => write!(f, "Victim's elevation"),
            FilterType::Distance2DFilter => write!(f, "2D distance"),
            FilterType::Distance3DFilter => write!(f, "3D distance"),
            FilterType::RoundFilter => write!(f, "Round #"),
            FilterType::PropertyFilter => write!(f, "Death property"),
        }
    }
}

pub struct FiltersPane {
    pub theme: style::Theme,
    pub busy: bool,
    pub filters: Vec<FilterRow>,
    scroll_state: scrollable::State,
    add_filter_button_state: button::State,
}

impl Default for FiltersPane {
    fn default() -> Self {
        let mut no_suicides_by_default = FilterRow {
            selected_filter: FilterType::PropertyFilter,
            selected_property: Property::Suicide,
            selected_property_operator: PropertyOperator::IsNotPresent,
            ..Default::default()
        };
        no_suicides_by_default.filter = no_suicides_by_default.try_generate_filter();
        let mut only_in_round_kills_by_default = FilterRow {
            selected_filter: FilterType::PropertyFilter,
            selected_property: Property::DuringRound,
            selected_property_operator: PropertyOperator::IsPresent,
            ..Default::default()
        };
        only_in_round_kills_by_default.filter = only_in_round_kills_by_default.try_generate_filter();
        Self {
            filters: vec![no_suicides_by_default, only_in_round_kills_by_default],
            theme: Default::default(),
            busy: Default::default(),
            scroll_state: Default::default(),
            add_filter_button_state: Default::default(),
        }
    }
}

impl FiltersPane {
    pub(crate) fn view(&mut self) -> Element<Message> {
        let theme = self.theme;
        let (filters, style): (Element<_>, _) = if self.filters.is_empty() {
            (
                Container::new(Text::new("No filter").width(Length::Fill).size(20).horizontal_alignment(HorizontalAlignment::Center))
                    .width(Length::Fill)
                    .center_y()
                    .into(),
                style::ResultContainer::Ok,
            )
        } else {
            let style = if self.filters.iter().all(|filter_row| filter_row.filter.is_some()) {
                style::ResultContainer::Ok
            } else {
                style::ResultContainer::Error
            };
            let col = self
                .filters
                .iter_mut()
                .enumerate()
                .fold(Column::new(), |col, (index, filter_row)| col.push(filter_row.view(index, theme)));
            (col.into(), style)
        };

        let add_filter_button = Button::new(&mut self.add_filter_button_state, Text::new("Add filter"))
            .style(self.theme)
            .on_press(Message::AddFilter);
        let header = Row::new().push(add_filter_button);
        let filters_scroll = Scrollable::new(&mut self.scroll_state).push(filters).width(Length::Fill).height(Length::Fill);
        let view = Column::new().push(header).push(filters_scroll);
        let result_container = Container::new(view).width(Length::Fill).height(Length::Fill).center_x().center_y().padding(10).style(style);

        Container::new(result_container).padding(4).width(Length::Fill).height(Length::Fill).into()
    }
}

#[derive(Default, Debug)]
pub struct FilterRow {
    pub filter: Option<Filter>,
    pub delete_button: button::State,
    pub filter_pick_list: pick_list::State<FilterType>,
    pub selected_filter: FilterType,
    pub class_button_state_scout: button::State,
    pub class_button_state_sniper: button::State,
    pub class_button_state_soldier: button::State,
    pub class_button_state_demoman: button::State,
    pub class_button_state_medic: button::State,
    pub class_button_state_heavy: button::State,
    pub class_button_state_pyro: button::State,
    pub class_button_state_spy: button::State,
    pub class_button_state_engineer: button::State,
    pub class_buttons_selected: [bool; 10],
    pub team_button_blu: button::State,
    pub team_button_red: button::State,
    pub team_button_selected: Team,
    pub ordered_operator_pick_list: pick_list::State<OrderedOperator>,
    pub selected_ordered_operator: OrderedOperator,
    pub text_input_state: text_input::State,
    pub text_input: String,
    pub property_operator_pick_list: pick_list::State<PropertyOperator>,
    pub selected_property_operator: PropertyOperator,
    pub property_pick_list: pick_list::State<Property>,
    pub selected_property: Property,
}

impl FilterRow {
    fn view(&mut self, index: usize, theme: style::Theme) -> Element<Message> {
        let pick_list = PickList::new(&mut self.filter_pick_list, &FilterType::ALL[..], Some(self.selected_filter), move |selected| {
            Message::FilterSelected(index, selected)
        });

        let filter_options = match self.selected_filter {
            FilterType::KillerTeamFilter | FilterType::VictimTeamFilter => {
                let mut row = Row::new();
                row = row.push(Button::new(&mut self.team_button_blu, Text::new("BLU")).on_press(Message::BluTeamClicked(index)).style(
                    if self.team_button_selected == Team::Blu {
                        ActiveButtonHighlight::Highlighted
                    } else {
                        ActiveButtonHighlight::NotHighlighted
                    },
                ));
                row = row.push(Button::new(&mut self.team_button_red, Text::new("RED")).on_press(Message::RedTeamClicked(index)).style(
                    if self.team_button_selected == Team::Red {
                        ActiveButtonHighlight::Highlighted
                    } else {
                        ActiveButtonHighlight::NotHighlighted
                    },
                ));
                row
            }
            FilterType::KillerClassFilter | FilterType::VictimClassFilter => {
                let mut row = Row::new();
                row = row.push(
                    Button::new(&mut self.class_button_state_scout, icon(CLASS_ICONS_CHARS[1]))
                        .on_press(Message::ClassIconClicked(index, 1))
                        .style(if self.class_buttons_selected[1] {
                            ActiveButtonHighlight::Highlighted
                        } else {
                            ActiveButtonHighlight::NotHighlighted
                        }),
                );
                row = row.push(
                    Button::new(&mut self.class_button_state_soldier, icon(CLASS_ICONS_CHARS[3]))
                        .on_press(Message::ClassIconClicked(index, 3))
                        .style(if self.class_buttons_selected[3] {
                            ActiveButtonHighlight::Highlighted
                        } else {
                            ActiveButtonHighlight::NotHighlighted
                        }),
                );
                row = row.push(
                    Button::new(&mut self.class_button_state_pyro, icon(CLASS_ICONS_CHARS[7]))
                        .on_press(Message::ClassIconClicked(index, 7))
                        .style(if self.class_buttons_selected[7] {
                            ActiveButtonHighlight::Highlighted
                        } else {
                            ActiveButtonHighlight::NotHighlighted
                        }),
                );
                row = row.push(
                    Button::new(&mut self.class_button_state_demoman, icon(CLASS_ICONS_CHARS[4]))
                        .on_press(Message::ClassIconClicked(index, 4))
                        .style(if self.class_buttons_selected[4] {
                            ActiveButtonHighlight::Highlighted
                        } else {
                            ActiveButtonHighlight::NotHighlighted
                        }),
                );
                row = row.push(
                    Button::new(&mut self.class_button_state_heavy, icon(CLASS_ICONS_CHARS[6]))
                        .on_press(Message::ClassIconClicked(index, 6))
                        .style(if self.class_buttons_selected[6] {
                            ActiveButtonHighlight::Highlighted
                        } else {
                            ActiveButtonHighlight::NotHighlighted
                        }),
                );
                row = row.push(
                    Button::new(&mut self.class_button_state_engineer, icon(CLASS_ICONS_CHARS[9]))
                        .on_press(Message::ClassIconClicked(index, 9))
                        .style(if self.class_buttons_selected[9] {
                            ActiveButtonHighlight::Highlighted
                        } else {
                            ActiveButtonHighlight::NotHighlighted
                        }),
                );
                row = row.push(
                    Button::new(&mut self.class_button_state_medic, icon(CLASS_ICONS_CHARS[5]))
                        .on_press(Message::ClassIconClicked(index, 5))
                        .style(if self.class_buttons_selected[5] {
                            ActiveButtonHighlight::Highlighted
                        } else {
                            ActiveButtonHighlight::NotHighlighted
                        }),
                );
                row = row.push(
                    Button::new(&mut self.class_button_state_sniper, icon(CLASS_ICONS_CHARS[2]))
                        .on_press(Message::ClassIconClicked(index, 2))
                        .style(if self.class_buttons_selected[2] {
                            ActiveButtonHighlight::Highlighted
                        } else {
                            ActiveButtonHighlight::NotHighlighted
                        }),
                );
                row = row.push(
                    Button::new(&mut self.class_button_state_spy, icon(CLASS_ICONS_CHARS[8]))
                        .on_press(Message::ClassIconClicked(index, 8))
                        .style(if self.class_buttons_selected[8] {
                            ActiveButtonHighlight::Highlighted
                        } else {
                            ActiveButtonHighlight::NotHighlighted
                        }),
                );
                row
            }
            FilterType::KillerElevationFilter | FilterType::VictimElevationFilter | FilterType::Distance2DFilter | FilterType::Distance3DFilter | FilterType::RoundFilter => {
                let pick_list = PickList::new(
                    &mut self.ordered_operator_pick_list,
                    &OrderedOperator::ALL[..],
                    Some(self.selected_ordered_operator),
                    move |selected| Message::OrderedOperatorSelected(index, selected),
                );
                let text_input = TextInput::new(&mut self.text_input_state, "value", &self.text_input, move |selected| {
                    Message::FilterTextInputChanged(index, selected)
                })
                .size(30)
                .style(theme);
                Row::new().push(pick_list).push(text_input)
            }
            FilterType::PropertyFilter => {
                let property_operator_pick_list = PickList::new(
                    &mut self.property_operator_pick_list,
                    &PropertyOperator::ALL[..],
                    Some(self.selected_property_operator),
                    move |selected| Message::PropertyOperatorSelected(index, selected),
                );
                let property_pick_list = PickList::new(&mut self.property_pick_list, &Property::ALL[..], Some(self.selected_property), move |selected| {
                    Message::PropertySelected(index, selected)
                });
                Row::new().push(property_pick_list).push(property_operator_pick_list)
            }
        };

        let delete_button = Button::new(&mut self.delete_button, delete_icon()).style(theme).on_press(Message::FilterRemoved(index));
        let row = Row::new().push(delete_button).push(pick_list).push(filter_options);
        let container_style = if self.filter.is_some() {
            style::ResultContainer::Ok
        } else {
            style::ResultContainer::Error
        };
        let result_container = Container::new(row).width(Length::Fill).center_y().padding(4).style(container_style).into();
        result_container
    }

    pub fn try_generate_filter(&mut self) -> Option<Filter> {
        match self.selected_filter {
            FilterType::KillerTeamFilter => Some(
                KillerTeamFilter {
                    team: match self.team_button_selected {
                        Team::Red => Team::Red,
                        Team::Blu => Team::Blu,
                        _ => return None,
                    },
                }
                .into(),
            ),
            FilterType::VictimTeamFilter => Some(
                VictimTeamFilter {
                    team: match self.team_button_selected {
                        Team::Red => Team::Red,
                        Team::Blu => Team::Blu,
                        _ => return None,
                    },
                }
                .into(),
            ),
            FilterType::KillerClassFilter => Some(
                KillerClassFilter {
                    classes: if self.class_buttons_selected.iter().any(|&b| b) {
                        self.class_buttons_selected
                    } else {
                        [false, true, true, true, true, true, true, true, true, true]
                        // none selected = all selected
                    },
                }
                .into(),
            ),
            FilterType::VictimClassFilter => Some(
                VictimClassFilter {
                    classes: if self.class_buttons_selected.iter().any(|&b| b) {
                        self.class_buttons_selected
                    } else {
                        [false, true, true, true, true, true, true, true, true, true]
                        // none selected = all selected
                    },
                }
                .into(),
            ),
            FilterType::KillerElevationFilter => Some(
                KillerElevationFilter {
                    op: self.selected_ordered_operator,
                    z: match self.text_input.parse() {
                        Ok(value) => value,
                        Err(_) => return None,
                    },
                }
                .into(),
            ),
            FilterType::VictimElevationFilter => Some(
                VictimElevationFilter {
                    op: self.selected_ordered_operator,
                    z: match self.text_input.parse() {
                        Ok(value) => value,
                        Err(_) => return None,
                    },
                }
                .into(),
            ),
            FilterType::Distance2DFilter => Some(
                Distance2DFilter {
                    op: self.selected_ordered_operator,
                    distance: match self.text_input.parse() {
                        Ok(value) => value,
                        Err(_) => return None,
                    },
                }
                .into(),
            ),
            FilterType::Distance3DFilter => Some(
                Distance3DFilter {
                    op: self.selected_ordered_operator,
                    distance: match self.text_input.parse() {
                        Ok(value) => value,
                        Err(_) => return None,
                    },
                }
                .into(),
            ),
            FilterType::RoundFilter => Some(
                RoundFilter {
                    op: self.selected_ordered_operator,
                    round: match self.text_input.parse() {
                        Ok(value) => value,
                        Err(_) => return None,
                    },
                }
                .into(),
            ),
            FilterType::PropertyFilter => Some(
                PropertyFilter {
                    op: self.selected_property_operator,
                    property: self.selected_property,
                }
                .into(),
            ),
        }
    }
}
