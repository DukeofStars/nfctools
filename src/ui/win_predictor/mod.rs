use std::fmt::Display;

use dioxus::prelude::*;

use crate::include_style;

#[component]
pub fn WinPredictor() -> Element {
    return rsx! {
        {include_style!("assets/main.css")}
        {include_style!("assets/dx-components-theme.css")}
        WinPredictorInner {}
    };
}

const ALPHABET: &str = "ABCDEFGHIJKJMNOPQRSTUVWXYZ";

#[derive(Debug, Clone, Copy, PartialEq)]
enum Team {
    Neutral,
    Team1,
    Team2,
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Team::Neutral => "team-neutral",
                Team::Team1 => "team1",
                Team::Team2 => "team2",
            }
        )
    }
}

#[component]
fn WinPredictorInner() -> Element {
    let mut num_caps = use_signal(|| 5usize);
    let mut cap_status = use_signal(|| [Team::Neutral; 10]);
    let mut max_points = use_signal(|| 1000usize);

    let mut team1_points = use_signal(|| 0usize);
    let mut team2_points = use_signal(|| 0usize);

    let mut size = use_signal(|| (0i32, 0i32));

    let mut winning_team = use_signal(|| Team::Neutral);
    let winning_ticks = use_memo(move || {
        let num_caps = num_caps();
        let caps = cap_status();
        let mut team1_caps = 0;
        let mut team2_caps = 0;
        for i in 0..num_caps {
            match caps[i] {
                Team::Neutral => {}
                Team::Team1 => team1_caps += 1,
                Team::Team2 => team2_caps += 1,
            }
        }

        if team1_caps == 0 && team2_caps == 0 {
            winning_team.set(Team::Neutral);
            return 0;
        }

        let mut team1 = team1_points();
        let mut team2 = team2_points();
        let mut num_ticks = 0;
        let max_points = max_points();
        loop {
            if team1 >= max_points && team2 >= max_points {
                winning_team.set(Team::Neutral);
                return num_ticks;
            } else if team1 >= max_points {
                winning_team.set(Team::Team1);
                return num_ticks;
            } else if team2 >= max_points {
                winning_team.set(Team::Team2);
                return num_ticks;
            }
            num_ticks += 1;
            team1 += team1_caps * 2;
            team2 += team2_caps * 2;
        }
    });

    rsx! {
        div { display: "flex", flex_direction: "column", overflow: "hidden",
            h1 { "Win Predictor" }
            input {
                r#type: "range",
                class: "cap-slider",
                min: "1",
                max: "10",
                value: "{num_caps}",
                width: "100%",
                onchange: move |evt| { num_caps.set(evt.value().parse().unwrap()) },
            }
            div { id: "cap-button-container",
                for cap in 0usize..num_caps() {
                    {
                        let char = ALPHABET.chars().nth(cap).unwrap();
                        rsx! {
                            button {
                                class: format!("cap-button button {}", cap_status()[cap]),
                                onclick: move |_| {
                                    cap_status.write()[cap] = match cap_status()[cap] {
                                        Team::Neutral => Team::Team1,
                                        Team::Team1 => Team::Team2,
                                        Team::Team2 => Team::Neutral,
                                    };
                                },
                                // Right click to cycle backwards
                                oncontextmenu: move |evt| {
                                    evt.prevent_default();
                                    cap_status.write()[cap] = match cap_status()[cap] {
                                        Team::Neutral => Team::Team2,
                                        Team::Team1 => Team::Neutral,
                                        Team::Team2 => Team::Team1,
                                    };
                                },
                                "{char}"
                            }
                        }
                    }
                }
            }
            div {
                width: "100%",
                display: "grid",
                // flex_grow: 1,
                overflow: "hidden",
                grid_template_columns: "60% 40%",
                onresize: move |evt| {
                    let rect = evt.data.get_border_box_size().unwrap_or_default();
                    size.set((rect.width as i32, rect.height as i32));
                },
                div {
                    PointsChart {
                        // For re-rendering on resize
                        size: size(),
                        num_caps: num_caps(),
                        caps: cap_status(),
                        team1_points_initial: team1_points(),
                        team2_points_initial: team2_points(),
                        max_points: max_points(),
                    }
                }
                div {

                    div {
                        height: "fit-content",
                        padding_right: "2px",
                        display: "grid",
                        grid_template_columns: "50% 50%",
                        "Max Points"
                        input {
                            value: "1000",
                            onchange: move |evt| { max_points.set(evt.value().parse().unwrap_or(1000)) },
                        }
                        "Team 1 Points"
                        input {
                            value: "0",
                            onchange: move |evt| { team1_points.set(evt.value().parse().unwrap_or_default()) },
                        }
                        "Team 2 Points"
                        input {
                            value: "0",
                            onchange: move |evt| { team2_points.set(evt.value().parse().unwrap_or_default()) },
                        }
                    }
                    match winning_team() {
                        Team::Neutral => "It's a Tie!".to_string(),
                        Team::Team1 => {
                            format!(
                                "Team 1 wins in {winning_ticks} ticks ({} seconds)",
                                winning_ticks * 10,
                            )
                        }
                        Team::Team2 => {
                            format!(
                                "Team 2 wins in {winning_ticks} ticks ({} seconds)",
                                winning_ticks * 10,
                            )
                        }
                    }
                }
            }
        }
    }
}

use dioxus_charts::LineChart;

#[component]
fn PointsChart(
    size: (i32, i32),
    num_caps: usize,
    caps: [Team; 10],
    team1_points_initial: usize,
    team2_points_initial: usize,
    max_points: usize,
) -> Element {
    let mut team1_caps = 0;
    let mut team2_caps = 0;
    for i in 0..num_caps {
        match caps[i] {
            Team::Neutral => {}
            Team::Team1 => team1_caps += 1,
            Team::Team2 => team2_caps += 1,
        }
    }

    let labels = (1..=30).map(|n| format!("{n}")).collect::<Vec<_>>();
    let team1_points = (0..=30)
        .map(|n| team1_points_initial + team1_caps * n * 12)
        .map(|n| n.clamp(0, max_points))
        .map(|n| n as f32)
        .collect::<Vec<_>>(); // 2 points per cap each 10 seconds, so 6*2 points per minute
    let team2_points = (0..=30)
        .map(|n| team2_points_initial + team2_caps * n * 12)
        .map(|n| n.clamp(0, max_points))
        .map(|n| n as f32)
        .collect::<Vec<_>>();

    rsx! {
        LineChart {
            padding_top: 30,
            padding_left: 20,
            padding_right: 10,
            padding_bottom: 30,
            viewbox_width: 200,
            viewbox_height: 200,
            show_dots: false,
            lowest: 0.0,
            highest: max_points as f32,
            series: vec![
                                                                                                                                                                                                                                                    team1_points,
                                                                                                                                                                                                                                                    team2_points,
                                                                                                                                                                                                                                                ],
            labels,
        }
    }
}
