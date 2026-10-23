// SPDX-License-Identifier: Apache-2.0

//! `deecure fix`, `deecure history`, and `deecure undo`. Fixes run in this terminal's
//! session, so polkit can ask for the password for admin steps.

use std::io::{BufRead, Write};
use std::path::Path;
use std::process::ExitCode;

use deecure_core::client;
use deecure_core::fix::{ACTIONS, FixRun, Plan, RunStatus, Undo};
use deecure_core::rpc::method;
use serde_json::json;

fn runtime() -> Option<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()
}

fn fail(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("deecure: {message}");
    ExitCode::FAILURE
}

fn size(bytes: u64) -> String {
    let gb = bytes as f64 / 1024f64.powi(3);
    if gb >= 1.0 {
        format!("{gb:.1} GB")
    } else {
        format!("{:.0} MB", bytes as f64 / 1024f64.powi(2))
    }
}

pub fn fix(
    socket: &Path,
    action: Option<String>,
    target: Option<String>,
    yes: bool,
    list: bool,
) -> ExitCode {
    let Some(action) = action.filter(|_| !list) else {
        println!("Fixes (add a target where shown):");
        for (id, title, target) in ACTIONS {
            let target = target.map(|t| format!(" <{t}>")).unwrap_or_default();
            println!("  {id}{target}: {title}");
        }
        return ExitCode::SUCCESS;
    };
    let plan: Plan = match client::call(
        socket,
        method::FIX_PLAN,
        Some(json!({ "action": action, "subject": target })),
    ) {
        Ok(p) => p,
        Err(e) => return fail(e),
    };
    println!(
        "{}{}",
        plan.title,
        if plan.subtitle.is_empty() {
            String::new()
        } else {
            format!(" · {}", plan.subtitle)
        }
    );
    if plan.steps.is_empty() {
        println!("Nothing to do right now.");
        return ExitCode::SUCCESS;
    }
    for s in &plan.steps {
        let undo = match s.undo {
            Undo::Full => "reversible",
            Undo::Regenerates => "regenerates",
            Undo::None => "IRREVERSIBLE",
        };
        let admin = if s.call.admin() { ", admin" } else { "" };
        let freed = s
            .size
            .map(|b| format!(", frees {}", size(b)))
            .unwrap_or_default();
        println!(
            "  • {} ({undo}{admin}{freed})\n      {}\n      $ {}",
            s.title, s.description, s.command
        );
    }
    if !yes {
        print!("Run these {} steps? [y/N] ", plan.steps.len());
        let _ = std::io::stdout().flush();
        let mut answer = String::new();
        let _ = std::io::stdin().lock().read_line(&mut answer);
        if !matches!(answer.trim(), "y" | "Y" | "yes") {
            println!("Nothing was changed.");
            return ExitCode::SUCCESS;
        }
    }
    let Some(rt) = runtime() else {
        return fail("could not start");
    };
    let chosen: Vec<String> = plan.steps.iter().map(|s| s.id.clone()).collect();
    let run = rt.block_on(deecure_fix::apply(&plan, &chosen, |r| {
        println!(
            "  {} {}: {}",
            if r.ok { "✓" } else { "✗" },
            r.title,
            r.message
        )
    }));
    match client::call::<i64>(socket, method::FIX_RECORD, Some(json!({ "run": run }))) {
        Ok(id) => println!(
            "Saved as #{id}. {}",
            if run.undo_calls().is_empty() {
                "It cannot be undone."
            } else {
                "Undo it with: deecure undo <number>"
            }
            .replace("<number>", &id.to_string())
        ),
        Err(e) => eprintln!("deecure: the fix ran, but could not be saved: {e}"),
    }
    if run.status == RunStatus::Succeeded {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

pub fn history(socket: &Path) -> ExitCode {
    let runs: Vec<FixRun> = match client::call(socket, method::FIX_RUNS, None) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    if runs.is_empty() {
        println!("DeeCure has not changed anything on this computer.");
    }
    for r in runs {
        let status = match r.status {
            RunStatus::Succeeded => "succeeded",
            RunStatus::Failed => "failed",
            RunStatus::Undone => "undone",
        };
        let target = r
            .subject
            .as_deref()
            .map(|s| format!(" · {s}"))
            .unwrap_or_default();
        let undo = if r.status != RunStatus::Undone && !r.undo_calls().is_empty() {
            " · can be undone"
        } else {
            ""
        };
        println!("#{:<4} {}{target}: {status}{undo}", r.id, r.title);
    }
    ExitCode::SUCCESS
}

pub fn undo(socket: &Path, id: i64) -> ExitCode {
    let runs: Vec<FixRun> = match client::call(socket, method::FIX_RUNS, None) {
        Ok(r) => r,
        Err(e) => return fail(e),
    };
    let Some(run) = runs.into_iter().find(|r| r.id == id) else {
        return fail(format!("no fix #{id}"));
    };
    if run.status == RunStatus::Undone {
        return fail(format!("#{id} is already undone"));
    }
    let Some(rt) = runtime() else {
        return fail("could not start");
    };
    match rt.block_on(deecure_fix::undo(&run)) {
        Ok(done) => {
            for m in done {
                println!("  ✓ {m}");
            }
            match client::call::<bool>(socket, method::FIX_UNDONE, Some(json!({ "id": id }))) {
                Ok(_) => println!("#{id} undone."),
                Err(e) => eprintln!("deecure: undone, but not saved: {e}"),
            }
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}
