use bpaf::{construct, env, long, positional, short, OptionParser, Parser};

/// Indicates what Tasks(s) to select
#[derive(Debug, Clone)]
pub enum TaskSelector {
    Indexed(Vec<usize>),
    All,
    Completed,
}

/// The command to execute
#[derive(Debug, Clone)]
pub enum Command {
    Show,
    NewProject { name: Option<String>, force: bool },
    AddTask { desc: String, completed: bool },
    MarkCompletion(bool, TaskSelector),
    RemoveTask(TaskSelector),
    EditTask(String, usize),
    PrintCompletion(String),
    RemoveProject,
}

fn options() -> OptionParser<Command> {
    let new_cmd = new_project_command()
        .command("new")
        .help("create a new project");

    let add_cmd = add_task_command()
        .command("add")
        .short('a')
        .help("add a new task");

    let done_cmd = task_completed_command()
        .command("done")
        .short('d')
        .help("mark a task as being completed");

    let rm_cmd = remove_task_command().command("rm").help("remove a task");

    let edit_cmd = edit_task_command()
        .command("edit")
        .short('e')
        .help("edit an existing task");

    let completion_cmd = print_completions_command()
        .command("completions")
        .help("print shell completions");

    construct!([new_cmd, add_cmd, done_cmd, rm_cmd, edit_cmd, completion_cmd])
        .fallback(Command::Show)
        .to_options()
        .version(concat!("tutel v", env!("CARGO_PKG_VERSION")))
        .descr("tutel\na minimalistic todo app for terminal enthusiasts")
        .footer("run without a subcommand to show the todo list")
}

#[test]
fn check_bpaf_invariants() {
    options().check_invariants(true)
}

/// Parse the command line and return the command to be executed
pub fn parse_cli() -> Command {
    options().run()
}

fn new_project_command() -> OptionParser<Command> {
    let name = positional::<String>("name").optional();
    let force = short('f')
        .long("force")
        .help("force project creation")
        .switch();

    construct!(Command::NewProject { force, name })
        .to_options()
        .descr("create a new project in the current directory")
}

fn add_task_command() -> OptionParser<Command> {
    let desc = positional::<String>("description")
        .many()
        .guard(|v| !v.is_empty(), "the task description is required")
        .map(|v| {
            let mut desc = String::new();
            let vlen = v.len();

            for (i, s) in v.iter().enumerate() {
                desc.push_str(s);
                if i < vlen - 1 {
                    desc.push(' ');
                }
            }
            desc
        });

    let completed = short('c')
        .long("completed")
        .help("mark the task as already completed")
        .switch();

    construct!(Command::AddTask { completed, desc })
        .to_options()
        .descr("add a new task. aliases: a")
}

fn task_completed_command() -> OptionParser<Command> {
    let completed = short('!')
        .long("not")
        .help("mark the task as not being done")
        .flag(false, true);

    // can unconditionally be mapped to TaskSelector::All since its value is only used if it is
    // present
    let all = short('a')
        .long("all")
        .help("select all tasks")
        .req_flag(TaskSelector::All);

    let selector = construct!([parse_indices(), all]);
    construct!(Command::MarkCompletion(completed, selector))
        .to_options()
        .descr("mark a task as being done. aliases: d")
}

fn remove_task_command() -> OptionParser<Command> {
    let all = short('a')
        .long("all")
        .help("remove all tasks")
        .req_flag(TaskSelector::All);

    let cleanup = short('c')
        .long("cleanup")
        .help("remove all completed tasks")
        .req_flag(TaskSelector::Completed);

    let project = long("project")
        .help("remove the whole project file")
        .req_flag(Command::RemoveProject);

    let remove_task = construct!([parse_indices(), all, cleanup]).map(Command::RemoveTask);

    construct!([remove_task, project])
        .to_options()
        .descr("remove a task from a project")
}

fn complete_indices(input: &Vec<String>) -> Vec<(String, Option<String>)> {
    let p = tutel::load_project_rec(&*std::env::current_dir().unwrap()).unwrap();
    let mut res = Vec::new();

    let full = &input[..input.len() - 1];
    let active = input.last().unwrap();

    for task in p.data.tasks {
        let tid = task.index.to_string();
        if full.contains(&tid) {
            continue;
        }
        if tid.starts_with(active) {
            res.push((format!("{}", task.index), Some(task.desc.clone())));
        }
    }

    res
}

fn parse_indices() -> impl Parser<TaskSelector> {
    positional::<String>("indices")
        .some("one or more task indices are required")
        .complete(complete_indices)
        .parse::<_, _, String>(|v| {
            let mut indices = Vec::with_capacity(v.len());

            for x in v {
                indices.push(
                    x.parse::<usize>()
                        .map_err(|_| format!("not a valid index: {x}"))?,
                )
            }

            Ok(TaskSelector::Indexed(indices))
        })
}

fn edit_task_command() -> OptionParser<Command> {
    let index = positional::<usize>("index");

    let editor = env("EDITOR")
        .short('e')
        .long("editor")
        .help("the editor to use (default: $EDITOR)")
        .argument::<String>("editor");

    construct!(Command::EditTask(editor, index))
        .to_options()
        .descr("edit an existing task. aliases: e")
}

fn print_completions_command() -> OptionParser<Command> {
    let shell = positional::<String>("shell");

    construct!(Command::PrintCompletion(shell))
        .to_options()
        .descr("print shell completions for the given shell")
}
