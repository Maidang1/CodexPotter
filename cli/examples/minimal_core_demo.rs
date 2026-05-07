use std::collections::VecDeque;
use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::Context;

const FIXED_TURN_PROMPT: &str = "Continue working according to the WORKFLOW_INSTRUCTIONS";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Initial,
    Open,
    Skip,
}

#[derive(Debug, Clone)]
struct ProgressFile {
    status: Status,
    finite_incantatem: bool,
    short_title: String,
    overall_goal: String,
    in_progress: Vec<String>,
    todo: Vec<String>,
    done: Vec<String>,
}

impl ProgressFile {
    fn new(overall_goal: String) -> Self {
        Self {
            status: Status::Initial,
            finite_incantatem: false,
            short_title: String::new(),
            overall_goal,
            in_progress: Vec::new(),
            todo: Vec::new(),
            done: Vec::new(),
        }
    }

    fn render(&self) -> String {
        let mut out = String::new();
        let status = match self.status {
            Status::Initial => "initial",
            Status::Open => "open",
            Status::Skip => "skip",
        };
        let _ = writeln!(out, "---");
        let _ = writeln!(out, "status: {status}");
        let _ = writeln!(out, "finite_incantatem: {}", self.finite_incantatem);
        let _ = writeln!(out, "short_title: {}", self.short_title);
        let _ = writeln!(out, "---");
        let _ = writeln!(out);
        let _ = writeln!(out, "# Overall Goal");
        let _ = writeln!(out);
        let _ = writeln!(out, "{}", self.overall_goal);
        let _ = writeln!(out);
        let _ = writeln!(out, "## In Progress");
        for item in &self.in_progress {
            let _ = writeln!(out, "- {item}");
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "## Todo");
        for item in &self.todo {
            let _ = writeln!(out, "- {item}");
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "## Done");
        for item in &self.done {
            let _ = writeln!(out, "- {item}");
        }
        out
    }
}

#[derive(Debug)]
struct Project {
    id: String,
    progress_path: PathBuf,
    progress: ProgressFile,
    queued_prompts: VecDeque<String>,
}

fn persist_progress(project: &Project) -> anyhow::Result<()> {
    if let Some(parent) = project.progress_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(&project.progress_path, project.progress.render())
        .with_context(|| format!("write {}", project.progress_path.display()))
}

fn simulate_one_round(project: &mut Project, round: u32, round_total: u32) -> anyhow::Result<()> {
    println!(
        "\n== Project {} | Round {round}/{round_total} ==",
        project.id
    );
    println!("Prompt: {FIXED_TURN_PROMPT}");

    match project.progress.status {
        Status::Initial => {
            project.progress.short_title = "Minimal core demo".to_string();
            project.progress.status = Status::Open;
            project
                .progress
                .todo
                .push("Understand overall goal".to_string());
            project
                .progress
                .todo
                .push("Implement smallest useful change".to_string());
        }
        Status::Open => {
            if let Some(task) = project.progress.in_progress.pop() {
                project
                    .progress
                    .done
                    .push(format!("completed: {task} (round {round})"));
            } else if let Some(task) = project.progress.todo.pop() {
                project.progress.in_progress.push(task);
            }

            if project.progress.todo.is_empty() && project.progress.in_progress.is_empty() {
                project.progress.finite_incantatem = true;
            }
        }
        Status::Skip => {
            project.progress.finite_incantatem = true;
        }
    }

    persist_progress(project)?;
    Ok(())
}

fn run_project(mut project: Project, rounds: u32) -> anyhow::Result<VecDeque<String>> {
    persist_progress(&project)?;
    for round in 1..=rounds {
        simulate_one_round(&mut project, round, rounds)?;
        if project.progress.finite_incantatem {
            println!(
                "Project {} stops early: finite_incantatem = true",
                project.id
            );
            break;
        }
    }
    Ok(project.queued_prompts)
}

fn main() -> anyhow::Result<()> {
    let root = std::env::temp_dir().join("codex_potter_minimal_demo");
    std::fs::create_dir_all(&root).with_context(|| format!("create {}", root.display()))?;

    let mut queue = VecDeque::new();
    queue.push_back(Project {
        id: "P1".to_string(),
        progress_path: root.join("P1").join("MAIN.md"),
        progress: ProgressFile::new("Refactor query engine with minimal safe changes".to_string()),
        queued_prompts: VecDeque::from([String::from(
            "Write short follow-up report after implementation",
        )]),
    });

    let rounds = 4;
    let mut next_project_index = 2;

    while let Some(project) = queue.pop_front() {
        let queued_prompts = run_project(project, rounds)?;
        for prompt in queued_prompts {
            let id = format!("P{next_project_index}");
            next_project_index += 1;
            let progress_path = root.join(&id).join("MAIN.md");
            queue.push_back(Project {
                id,
                progress_path,
                progress: ProgressFile::new(prompt),
                queued_prompts: VecDeque::new(),
            });
        }
    }

    println!(
        "\nDemo finished. Progress files written under: {}",
        root.display()
    );
    Ok(())
}
