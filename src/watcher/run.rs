use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};
use tokio::time::sleep;

use crate::{
    context::Context,
    util::read_tx_stats,
    watcher::{
        format::{format_bps, format_pps},
        stats::Stats,
    },
};

/// Runs the interactive watcher interface that displays real-time TX stats for the specified network interface.
/// This function will continuously read TX bytes and packets from /proc/net/dev for the given interface, compute the packets per second (PPS) and bits per second (BPS), and update the terminal UI with this information. The watcher will run until the `running` flag is set to false, at which point it will clean up the terminal state and exit.
///
/// # Arguments
/// * `_ctx` - The application context, which may contain shared data and resources (not currently used in this function but included for potential future use).
/// * `running` - An atomic boolean flag that indicates whether the watcher should continue running. The watcher will exit when this flag is set to false.
/// * `iface` - The name of the network interface to monitor (e.g., "eth0"). The function will read TX stats for this interface from /proc/net/dev.
///
/// # Returns
/// A `Result` indicating success or failure. If the function encounters an error while setting up the terminal or reading stats, it will return an `anyhow::Error`.
pub async fn watcher_run(_ctx: Context, running: Arc<AtomicBool>, iface: String) -> Result<()> {
    // We'll want to enable raw amode.
    enable_raw_mode()?;

    // Grab stdout and enter alternate screen for drawing.
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Hide the cursor while in the watcher.
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear the cursor so there isn't any overlapping text from before (cleanup)
    terminal.clear()?;

    // Initialize our TX stats.
    let mut stats = Stats::new(60); // 60 seconds of history

    let (init_bytes, init_packets) = read_tx_stats(&iface)?;

    let mut prev_tx_bytes = init_bytes;
    let mut prev_tx_packets = init_packets;

    let mut t = 0.0f64;

    loop {
        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Check for Ctrl+C keypress inside raw mode
        // We need this even though we have the tokio::select! signal in the main thread.
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    running.store(false, Ordering::Relaxed);

                    break;
                }
            }
        }

        // Read current TX stats and compute PPS and BPS from the difference since the last read.
        let (tx_bytes, tx_packets) = read_tx_stats(&iface)?;

        let pps = (tx_packets.saturating_sub(prev_tx_packets)) as f64;
        let bps = (tx_bytes.saturating_sub(prev_tx_bytes)) as f64 * 8.0;

        // Update previous stats for the next iteration.
        prev_tx_packets = tx_packets;
        prev_tx_bytes = tx_bytes;

        // Push the new stats into our history for graphing.
        stats.push(t, pps, bps);
        t += 1.0;

        // Compute x bounds from history
        let x_min = stats.pps_history.first().map(|p| p.0).unwrap_or(0.0);
        let x_max = x_min + stats.max_points as f64;

        // Compute y bounds for PPS and BPS with some padding.
        let pps_max = stats
            .pps_history
            .iter()
            .map(|p| p.1 as u64)
            .max()
            .unwrap_or(1) as f64
            * 1.2;

        let bps_max = stats
            .bps_history
            .iter()
            .map(|p| p.1 as u64)
            .max()
            .unwrap_or(1) as f64
            * 1.2;

        terminal.draw(|f| {
            let area = f.area();

            // Create chunks for current stats header, PPS chart, and BPS chart. The header will be a fixed height and the charts will split the remaining space.
            // Admittedly I used AI for this LOL, but I'm trying to learn Ratatui as well!
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),      // header with current stats
                    Constraint::Percentage(50), // PPS chart
                    Constraint::Percentage(50), // BPS chart
                ])
                .split(area);

            // Retrieve the current stats entry for real-time display.
            let current_pps = stats.pps_history.last().map(|p| p.1).unwrap_or(0.0);
            let current_bps = stats.bps_history.last().map(|p| p.1).unwrap_or(0.0);

            // Format read-time stats.
            let header = Paragraph::new(format!(
                "PPS: {}    Throughput: {}",
                format_pps(current_pps),
                format_bps(current_bps * 8.0), // bytes -> bits
            ))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Current Stats"),
            )
            .style(Style::default().fg(Color::Yellow));

            f.render_widget(header, chunks[0]);

            // Construct and render PPS chart.
            let pps_data = Dataset::default()
                .name("PPS")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&stats.pps_history);

            let pps_chart = Chart::new(vec![pps_data])
                .block(
                    Block::default()
                        .title("Packets Per Second")
                        .borders(Borders::ALL),
                )
                .x_axis(Axis::default().bounds([x_min, x_max]).labels(vec![
                    Span::raw(format!("{:.0}s", x_min)),
                    Span::raw(format!("{:.0}s", x_max)),
                ]))
                .y_axis(Axis::default().bounds([0.0, pps_max]).labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{:.0}M", pps_max / 1_000_000.0)),
                ]));

            f.render_widget(pps_chart, chunks[1]);

            // Construct and render BPS chart.
            let bps_data = Dataset::default()
                .name("BPS")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&stats.bps_history);

            let bps_chart = Chart::new(vec![bps_data])
                .block(
                    Block::default()
                        .title("Bits Per Second")
                        .borders(Borders::ALL),
                )
                .x_axis(Axis::default().bounds([x_min, x_max]).labels(vec![
                    Span::raw(format!("{:.0}s", x_min)),
                    Span::raw(format!("{:.0}s", x_max)),
                ]))
                .y_axis(Axis::default().bounds([0.0, bps_max]).labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{:.2}G", bps_max / 1_000_000_000.0)),
                ]));

            f.render_widget(bps_chart, chunks[2]);
        })?;

        // Sleep 1 second for next update (per second updates).
        sleep(Duration::from_secs(1)).await;
    }

    // Disable raw mode now since we're done with the watcher.
    disable_raw_mode()?;

    // Restore cursor and leave alternate screen
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    Ok(())
}
