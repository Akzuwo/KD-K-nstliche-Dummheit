use anyhow::{Context, Result, bail, ensure};
use kd_desktop::{
    engine::{Event, Runtime},
    models,
    store::{self, Store},
};
use serde_json::json;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

fn arg(args: &[String], name: &str) -> Result<String> {
    args.windows(2)
        .find(|a| a[0] == name)
        .map(|a| a[1].clone())
        .with_context(|| format!("Argument fehlt: {name}"))
}
fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.get(1).map(String::as_str) {
        Some("package") => {
            models::package(
                &PathBuf::from(arg(&args, "--model")?),
                &PathBuf::from(arg(&args, "--output")?),
                &arg(&args, "--name")?,
                &arg(&args, "--version")?,
            )?;
            println!("Modellpaket erstellt und SHA-256 hinterlegt.");
        }
        Some("import") => {
            let root = PathBuf::from(arg(&args, "--data")?);
            let m = models::import(&root, &PathBuf::from(arg(&args, "--zip")?), |p| {
                println!("{p}")
            })?;
            println!("{}", serde_json::to_string_pretty(&m)?);
        }
        Some("live-test") => {
            tokio::runtime::Runtime::new()?.block_on(live_test(&args))?;
        }
        _ => bail!(
            "kd-tool package --model MODELL.gguf --output PAKET.zip --name KD --version 0.4.1\nkd-tool import --data DATENORDNER --zip PAKET.zip\nkd-tool live-test --data NEUER_TESTORDNER --zip PAKET.zip --runtime LAUFZEITORDNER"
        ),
    }
    Ok(())
}
async fn live_test(args: &[String]) -> Result<()> {
    let root = PathBuf::from(arg(args, "--data")?);
    ensure!(
        !root.exists(),
        "Live-Test benötigt einen neuen Datenordner."
    );
    let runtime = PathBuf::from(arg(args, "--runtime")?);
    let zip = PathBuf::from(arg(args, "--zip")?);
    let model = models::import(&root, &zip, |p| println!("{p}"))?;
    let store = Store::open(&root)?;
    store.set_setting("active_model", &model.id)?;
    let chat = store.create_chat("Live-Test: Mozart")?;
    let flag = AtomicBool::new(false);
    let mut answers = Vec::new();
    let start = Instant::now();
    let rt = Runtime::start(
        &runtime,
        &models::model_path(&root, &model),
        model.context,
        "cpu",
        &root.join("logs"),
        &flag,
    )
    .await?;
    let pid = rt.pid;
    for question in [
        "Wer war Wolfgang Amadeus Mozart?",
        "Welche Küche hat er erfunden?",
    ] {
        let mut history = store::context(&store.messages(&chat)?);
        history.push(json!({"role":"user","content":question}));
        store.add(&chat, "user", question, "complete", "")?;
        let id = store.add(&chat, "assistant", "", "streaming", &model.name)?;
        let mut text = String::new();
        let mut pieces = 0;
        let reason = rt
            .generate(history, 120, model.context, &flag, |event| {
                if let Event::Piece(p) = event {
                    text.push_str(&p);
                    pieces += 1;
                }
            })
            .await?;
        ensure!(
            pieces > 1 && !text.trim().is_empty(),
            "Echte Streaming-Ausgabe fehlt."
        );
        store.update(&id, &text, "complete")?;
        answers.push(json!({"question":question,"answer":text,"pieces":pieces,"finish":reason}));
        println!("Antwort mit {pieces} Streaming-Teilen erhalten.");
    }
    let mut partial = String::new();
    let mut count = 0;
    let abort_start = Instant::now();
    let stopped = rt
        .generate(
            vec![
                json!({"role":"user","content":"Erkläre die Geschichte der Schulen ausführlich."}),
            ],
            900,
            model.context,
            &flag,
            |event| {
                if let Event::Piece(p) = event {
                    partial.push_str(&p);
                    count += 1;
                    if count >= 8 {
                        flag.store(true, Ordering::SeqCst)
                    }
                }
            },
        )
        .await;
    ensure!(
        stopped.is_err() && count >= 8,
        "Abbruch wurde nicht wirksam."
    );
    drop(rt);
    let stopped_chat = store.create_chat("Abbruch")?;
    store.add(&stopped_chat, "user", "Schulen", "complete", "")?;
    store.add(&stopped_chat, "assistant", &partial, "stopped", &model.name)?;
    ensure!(
        store::context(&store.messages(&stopped_chat)?).is_empty(),
        "Teilantwort gelangt in Modellkontext."
    );
    flag.store(false, Ordering::SeqCst);
    let rt = Runtime::start(
        &runtime,
        &models::model_path(&root, &model),
        model.context,
        "cpu",
        &root.join("logs"),
        &flag,
    )
    .await?;
    let long = rt
        .generate(
            vec![json!({"role":"user","content":"sehr lang ".repeat(7000)})],
            900,
            model.context,
            &flag,
            |_| {},
        )
        .await;
    ensure!(long.is_err(), "Überlange Eingabe nicht zurückgewiesen.");
    let mut resumed = String::new();
    let finish = rt
        .generate(
            vec![json!({"role":"user","content":"Wie viel ist 6 + 7?"})],
            96,
            model.context,
            &flag,
            |e| {
                if let Event::Piece(p) = e {
                    resumed.push_str(&p)
                }
            },
        )
        .await?;
    ensure!(
        !resumed.trim().is_empty(),
        "Neue Antwort nach Abbruch fehlt."
    );
    drop(rt);
    store.rename(&chat, "Mozart lokal geprüft")?;
    ensure!(
        Store::open(&root)?.chats("Mozart")?.len() == 1,
        "Persistenz oder Suche defekt."
    );
    let report = json!({"passed":true,"model":model,"engine_pid":pid,"seconds":start.elapsed().as_secs_f64(),"answers":answers,"abort":{"pieces":count,"partial":partial,"seconds_including_restart":abort_start.elapsed().as_secs_f64()},"resume":{"text":resumed,"finish":finish},"checks":["ZIP import + SHA256","real CPU inference","live SSE streaming","multi-turn context","abort during generation","restart after abort","oversized context rejected","SQLite persistence/search/rename","partial response excluded"]});
    std::fs::write(
        root.join("live-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("PASS — {}", root.join("live-report.json").display());
    Ok(())
}
