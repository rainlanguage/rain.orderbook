use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState, Tabs, BorderType},
    Frame, Terminal, text::{Spans, Span},
};

use crate::subgraph::showorder::get_orders;

struct App {
    state: TableState,
    items: Vec<Vec<String>>,
} 

impl App {
    async fn new(uri : String) -> App { 
       
        let orders = get_orders(uri).await.unwrap() ; 
         
        App {
            state: TableState::default(),
            items: orders
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub async fn list_orders(uri : String) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    // let mut sp = Spinner::new(
    //     Spinners::from_str("Dots9").unwrap(),
    //     "Fetching Orders...".into(),
    // ); 
    let app = App::new(uri).await;
    // sp.stop(); 
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints(
            [
                Constraint::Percentage(90),
                Constraint::Percentage(10),
            ].as_ref()
        )
        .margin(5)
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Order Id", "Owner", "Input Vault Balance", "Output Vault Balance"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(&**c));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("ORDERS").border_style(Style::default().fg(Color::Cyan)).border_type(BorderType::Double))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(40),
            Constraint::Percentage(10),
            Constraint::Length(25),
            Constraint::Min(25),
        ]);
    f.render_stateful_widget(t.clone(), rects[0], &mut app.state); 
 
    let menu = vec![
        Spans::from(vec![
            Span::raw("⬆️ "),
        ]) ,
        Spans::from(vec![
            Span::raw("⬇️ "),
        ]) ,    
        Spans::from(vec![
            Span::styled("Q", Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED)),
            Span::styled("uit", Style::default().fg(Color::White)),
        ]) 
    ] ;

    let tabs = Tabs::new(menu)
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|")); 

    f.render_widget(tabs, rects[1])

    
}