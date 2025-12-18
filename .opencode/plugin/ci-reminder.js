// CI REMINDER PLUGIN
// Reminds to run CI checks before git commits

export const CiReminderPlugin = async ({ project, client, $, directory, worktree }) => {
  console.log("🔔 CI Reminder Plugin loaded - Will remind you to run CI checks")
  
  let hasRunChecks = false
  
  return {
    // Hook into tool execution to remind before git commits
    "tool.execute.before": async (input, output) => {
      // Check if this is a git commit command
      if (input.tool === "bash" && 
          (input.args.command.includes("git commit") || 
           input.args.command.includes("git push"))) {
        
        if (!hasRunChecks) {
          console.log("⚠️  CI REMINDER: Have you run CI checks before committing?")
          console.log("Run: cargo fmt --all -- --check")
          console.log("Run: cargo clippy --all -- -D warnings")
          console.log("Run: cargo check --all")
          console.log("Run: cargo test hybrid:: -- --test-threads=1")
          console.log("Or run the checkci tool: /checkci")
          console.log("")
          
          // Ask for confirmation (simulated - real implementation would use prompt)
          console.log("Type 'yes' to proceed anyway, or run checks first.")
          // In real plugin, we would use client.prompt() or similar
        }
      }
      
      // Allow the tool to proceed
      return output
    },
    
    // Mark checks as run when checkci tool is used
    "tool.execute.after": async (input, output) => {
      if (input.tool === "checkci" && output.includes("ALL CI CHECKS PASSED")) {
        hasRunChecks = true
        console.log("✅ CI checks marked as completed")
      }
      return output
    },
    
    // Provide checkci tool
    tool: {
      checkci: {
        description: "Run CI checks and mark them as completed",
        args: {},
        async execute(args, ctx) {
          console.log("🧪 Running CI checks...")
          
          const results = []
          
          try {
            // 1. Formatting
            console.log("📝 Checking formatting...")
            await ctx.$`cd ${ctx.directory} && cargo fmt --all -- --check`
            results.push("✅ Formatting OK")
            
            // 2. Clippy
            console.log("🔍 Running clippy...")
            await ctx.$`cd ${ctx.directory} && cargo clippy --all -- -D warnings`
            results.push("✅ Clippy OK")
            
            // 3. Compilation
            console.log("🔧 Checking compilation...")
            await ctx.$`cd ${ctx.directory} && cargo check --all`
            results.push("✅ Compilation OK")
            
            // 4. Tests
            console.log("🧪 Running tests...")
            await ctx.$`cd ${ctx.directory} && cargo test hybrid:: -- --test-threads=1`
            results.push("✅ Tests OK")
            
            hasRunChecks = true
            
            return "🎉 ALL CI CHECKS PASSED!\n" + results.join("\n")
            
          } catch (error) {
            return `❌ CI CHECKS FAILED: ${error.message}\n` + results.join("\n")
          }
        }
      }
    }
  }
}