use anyhow::{Result, anyhow, bail};
use std::{collections::VecDeque, sync::atomic::Ordering, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};
use tokio::time::{Instant, sleep};

use crate::{
    logger::level::LogLevel,
    util::{format_byt, format_pkt, read_tx_stats},
    watcher::base::{HISTORY_LEN, Watcher, WatcherMode},
};

struct ChartMeta {
    label: &'static str,
    color: Color,
}

const CHARTS: [ChartMeta; 2] = [
    ChartMeta {
        label: "Packets",
        color: Color::Red,
    },
    ChartMeta {
        label: "Bytes",
        color: Color::LightBlue,
    },
];

impl Watcher {
    pub async fn interface_start(&mut self) -> Result<()> {
        let iface: String = match self.ctx.batch.read().await.cur_batch_id {
            Some(batch_id) => {
                let batch = self.ctx.batch.read().await;

                let batch = match batch.batches.iter().find(|b| b.id == batch_id) {
                    Some(b) => b,
                    None => bail!(
                        "Current batch ID {} does not correspond to any batch",
                        batch_id
                    ),
                };

                let iface = batch.iface.clone().ok_or_else(|| {
                    anyhow!("Batch {} does not have an interface specified", batch_id)
                })?;

                iface
            }
            None => match self.iface_fb.clone() {
                Some(iface) => iface,
                None => bail!("No batch selected and no fallback interface provided"),
            },
        };

        let (init_pkts, init_byts) = read_tx_stats(&iface)?;

        let mut prev_tx_bytes = init_byts;
        let mut prev_tx_packets = init_pkts;

        let mut last_update = Instant::now();

        loop {
            if !self.ctx.running.load(Ordering::Relaxed) {
                break;
            }

            // Handle keypresses.
            if event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.ctx.running.store(false, Ordering::Relaxed);

                            break;
                        }
                        KeyCode::Char('1') => {
                            self.ctx
                                .logger
                                .read()
                                .await
                                .log_msg(LogLevel::Info, "Switched to Total mode")
                                .ok();

                            self.mode = WatcherMode::Total
                        }
                        KeyCode::Char('2') => {
                            self.ctx
                                .logger
                                .read()
                                .await
                                .log_msg(LogLevel::Info, "Switched to Rate mode")
                                .ok();

                            self.mode = WatcherMode::Rate
                        }
                        _ => (),
                    }
                }
            }

            // Determine if we're in rate mode for display purposes.
            let is_rate = self.mode == WatcherMode::Rate;

            // Get elapsed time since last stats read to calculate rates.
            let elapsed = (Instant::now() - last_update).as_secs_f64();

            let (tx_pkts_raw, tx_byts_raw) = read_tx_stats(&iface)?;

            let tx_pkts = tx_pkts_raw.saturating_sub(init_pkts);
            let tx_byts = tx_byts_raw.saturating_sub(init_byts);

            // Calculate rates if needed.
            let (pps, bps) = {
                let delta_pkt = tx_pkts.saturating_sub(prev_tx_packets);
                let delta_byt = tx_byts.saturating_sub(prev_tx_bytes);

                let pps = (delta_pkt as f64 / elapsed) as u64;
                let bps = (delta_byt as f64 / elapsed) as u64;

                if elapsed >= 1.0 {
                    // Only push to history if we've actually elapsed time to avoid skewing the charts with near-zero values.
                    push_history(&mut self.history_rate[0], pps as f64);
                    push_history(&mut self.history_rate[1], bps as f64);

                    prev_tx_bytes = tx_byts;
                    prev_tx_packets = tx_pkts;
                    last_update = Instant::now();
                }

                (pps, bps)
            };

            // Push total values.
            push_history(&mut self.history_tot[0], tx_pkts as f64);
            push_history(&mut self.history_tot[1], tx_byts as f64);

            self.ctx
                .logger
                .read()
                .await
                .log_msg(
                    LogLevel::Trace,
                    &format!(
                        "Stats::tx_pkts: {}, tx_byts: {}, pps: {}, bps: {}",
                        tx_pkts, tx_byts, pps, bps
                    ),
                )
                .ok();

            // Snapshot data for the draw closure.
            let history_tot = self.history_tot.clone();
            let history_rate = self.history_rate.clone();

            let logs_snapshot: Vec<String> = match &self.logs {
                Some(buffer) => buffer.lock().unwrap().iter().cloned().collect(),
                None => Vec::new(),
            };

            let stats_snapshot = if is_rate {
                (pps, bps)
            } else {
                (tx_pkts, tx_byts)
            };

            self.terminal.draw(|f| {
                let area = f.area();

                // Outer layout: header | charts | logs
                let outer = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(10),
                        Constraint::Length(8),
                    ])
                    .split(area);

                // Initialize header.
                let header_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(40), // Packets
                        Constraint::Percentage(40), // Bytes
                        Constraint::Percentage(20), // Mode indicator
                    ])
                    .split(outer[0]);

                let (pkt, byt) = stats_snapshot;

                // Draw top header with current values and mode.
                for (i, meta) in CHARTS.iter().enumerate() {
                    let val = if i == 0 {
                        format_pkt(pkt as f64, is_rate)
                    } else {
                        format_byt(byt as f64, is_rate)
                    };

                    let para = Paragraph::new(format!("{}: {}", meta.label, val))
                        .block(Block::default().borders(Borders::ALL))
                        .style(Style::default().fg(meta.color).add_modifier(Modifier::BOLD));

                    f.render_widget(para, header_chunks[i]);
                }

                // Add modes indicator at end of header (3rd chunk).
                let (mode_text, mode_color) = if is_rate {
                    ("[1] Tot [2] Rate", Color::Cyan)
                } else {
                    ("[1] Tot [2] Rate", Color::Yellow)
                };

                let mode_para = Paragraph::new(mode_text)
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD));
                f.render_widget(mode_para, header_chunks[2]);

                // Time to write the charts!
                let chart_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50); 2])
                    .split(outer[1]);

                // Active history depends on mode.
                let active_history = if is_rate { &history_rate } else { &history_tot };

                for (i, meta) in CHARTS.iter().enumerate() {
                    let data = to_chart_data(&active_history[i]);

                    // Calculate the Y axis depending on the minimum and maximum boundaries/values.
                    let y_min = active_history[i].iter().cloned().fold(f64::MAX, f64::min);
                    let y_min = if y_min == f64::MAX { 0.0 } else { y_min * 0.9 };

                    let y_max = active_history[i].iter().cloned().fold(0.0_f64, f64::max) * 1.1;
                    let y_max = if y_max == 0.0 { 1.0 } else { y_max };

                    // Prepare dataset for the chart.
                    let datasets = vec![
                        Dataset::default()
                            .marker(symbols::Marker::Braille)
                            .graph_type(GraphType::Line)
                            .style(Style::default().fg(meta.color))
                            .data(&data),
                    ];

                    // Choose formatting function based on whether we're in rate mode or not.
                    let fmt: fn(f64, bool) -> String = if i == 0 { format_pkt } else { format_byt };

                    let chart = Chart::new(datasets)
                        .block(
                            Block::default()
                                .title(Span::styled(
                                    meta.label,
                                    Style::default().fg(meta.color).add_modifier(Modifier::BOLD),
                                ))
                                .borders(Borders::ALL),
                        )
                        .x_axis(
                            Axis::default()
                                .bounds([0.0, HISTORY_LEN as f64])
                                .style(Style::default().fg(Color::DarkGray)),
                        )
                        .y_axis(
                            Axis::default()
                                .bounds([y_min, y_max])
                                .labels(vec![
                                    Span::raw(fmt(y_min, is_rate)),
                                    Span::raw(fmt((y_min + y_max) / 2.0, is_rate)),
                                    Span::raw(fmt(y_max, is_rate)),
                                ])
                                .style(Style::default().fg(Color::DarkGray)),
                        );

                    f.render_widget(chart, chart_chunks[i]);
                }

                // Prepare log text for display.
                let log_text: Vec<Line> = logs_snapshot
                    .iter()
                    .rev()
                    .take(6)
                    .map(|l| Line::from(l.as_str()))
                    .collect();

                let log_para = Paragraph::new(log_text)
                    .block(
                        Block::default()
                            .title("Logs")
                            .borders(Borders::ALL)
                            .style(Style::default().fg(Color::DarkGray)),
                    )
                    .style(Style::default().fg(Color::Gray));

                f.render_widget(log_para, outer[2]);
            })?;

            sleep(Duration::from_millis(self.refresh_rate)).await;
        }

        Ok(())
    }
}

fn push_history(dq: &mut VecDeque<f64>, val: f64) {
    if dq.len() >= HISTORY_LEN {
        dq.pop_front();
    }

    dq.push_back(val);
}

fn to_chart_data(dq: &VecDeque<f64>) -> Vec<(f64, f64)> {
    let offset = HISTORY_LEN.saturating_sub(dq.len());
    dq.iter()
        .enumerate()
        .map(|(i, &v)| ((i + offset) as f64, v))
        .collect()
}
