use std::env::args;

use regress::{Match, Regex};
const WORKER_OR_SHARED_WORKER_RE: &str = r#"(?:\?|&)(worker|sharedworker)(?:&|$)"#;
const WORKER_IMPORT_META_URL_RE: &str = r#"\bnew\s+(?:Worker|SharedWorker)\s*\(\s*(new\s+URL\s*\(\s*('[^']+'|"[^"]+"|`[^`]+`)\s*,\s*import\.meta\.url[^)]*\))"#;
#[test]
fn test_regex() {
  let re = Regex::new(WORKER_OR_SHARED_WORKER_RE).unwrap();
  let test_str = "src/worker/test.worker.ts?worker";
  assert_eq!(re.find(test_str).is_some(), true);

  //   let re = Regex::new(WORKER_IMPORT_META_URL_RE).unwrap();
  let test_str = r#"import type { Uri } from "vscode";
import type { Logger } from "monaco-languageclient/tools";
import { useWorkerFactory } from "monaco-editor-wrapper/workerFactory";
import { RegisteredMemoryFile } from "@codingame/monaco-vscode-files-service-override";
import type { IStoredWorkspace } from "@codingame/monaco-vscode-configuration-service-override";
import Editor from 'monaco-editor/esm/vs/editor/editor.worker?worker';
export const disableButton = (id: string, disabled: boolean) => {
	const button = document.getElementById(id) as HTMLButtonElement | null;
	if (button !== null) {
		button.disabled = disabled;
	}
};

export const configureMonacoWorkers = (logger?: Logger) => {
	useWorkerFactory({
		workerOverrides: {
			ignoreMapping: true,
			workerLoaders: {
				TextEditorWorker: () =>
					new Editor(),
				TextMateWorker: () =>
					new Worker(
						new URL("@codingame/monaco-vscode-textmate-service-override/worker",import.meta.url,asdasdas),{ type: "module" },
					),
			},
		},
		logger,
	});
};

export const createDefaultWorkspaceFile = (
	workspaceFile: Uri,
	workspacePath: string,
) => {
	return new RegisteredMemoryFile(
		workspaceFile,
		JSON.stringify(
			<IStoredWorkspace>{
				folders: [
					{
						path: workspacePath,
					},
				],
			},
			null,
			2,
		),
	);
};
"#;
  
  //   for c in re.find(&test_str).unwrap().groups() {
  //     println!("{:?}", &test_str[c.unwrap()]);
  //     // 我需要递归这个 test_str 后续的字符
  //   }

  // fn match_global(regex_str: &str, text: &str) -> Vec<Match> {
  //   let re = Regex::new(regex_str).unwrap();
  //   let mut matchs: Vec<Match> = Vec::new();
  //   let mut start = 0;
  //   loop {
  //     let m = re.find_from(text, start).next();
  //     match m {
  //       Some(m) => {
  //         matchs.push(m.clone());
  //         start = m.range().end;
  //         if start >= text.len() {
  //           break;
  //         }
  //       }
  //       None => break,
  //     }
  //   }
  //   matchs
  // }

  // let matches = match_global(WORKER_IMPORT_META_URL_RE, &test_str);
  let matches = Regex::new(WORKER_IMPORT_META_URL_RE).unwrap().find_iter(&test_str).collect::<Vec<Match>>();
  println!("matches : {:?}", matches);
  matches.iter().for_each(|m| {
    let args = &m.captures[0].clone().unwrap();
    let worker_url = &m.captures[1].clone().unwrap();
    println!("args:{}",&test_str[args.start..args.end]);
    println!("worker_url:{}",&test_str[worker_url.start..worker_url.end])
  });
}
