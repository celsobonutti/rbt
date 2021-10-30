interface Rbt
  exposes [ Rbt, init, Job, job, Command, exec, Tool, tool, systemTool ]
  imports []

# TODO: these are all out of order due to https://github.com/rtfeldman/roc/issues/1642. Once that's fixed, they should rearrange into the order in `exposes`

# TODO: how are we gonna get tools from Jobs? Maybe Tool, Command, and Job need to live in a single union and have private aliases outwards?
Tool : [ @Tool { name: Str } ]

systemTool : Str -> Tool
systemTool = \name ->
    @Tool { name }

Command : [ @Command { tool : Tool, args : List Str } ]

exec : Tool, List Str -> Command
exec = \execTool, args ->
    @Command { tool: execTool, args }

Job : [ @Job { command : Command, inputs : List Job, inputFiles : List Str, outputs : List Str } ]

# TODO: these fields are all required until https://github.com/rtfeldman/roc/issues/1844 is fixed
job : { command : Command, inputs : List Job, inputFiles : List Str, outputs : List Str } -> Job
job = \{ command, outputs, inputs, inputFiles } ->
    @Job { command, inputs, inputFiles, outputs }

Rbt : [ @Rbt { default : Job } ]

init : { default : Job } -> Rbt
init = \rbt -> @Rbt rbt

tool : Job, Str -> Tool
tool = \_, _ ->
    @Tool { name: "TODO" }
