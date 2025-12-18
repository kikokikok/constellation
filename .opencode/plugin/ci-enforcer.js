// CI ENFORCER PLUGIN
// Automatically runs CI checks before git commits
// Prevents commits that would fail CI

export const CiEnforcerPlugin = async ({ project, client, $, directory, worktree }) => {
  console.log("🔒 CI Enforcer Plugin loaded - Will enforce CI checks before commits")
  
  return {
    // Hook into tool execution to intercept git commits
    "tool.execute.before": async (input, output) => {
      // Check if this is a git commit command
      if (input.tool === "bash" && 
          (input.args.command.includes("git commit") || 
           input.args.command.includes("git push"))) {
        
        console.log("🚫 CI Enforcer: Detected git operation, running pre-commit checks...")
        
        try {
          // 1. Formatting check
          console.log("📝 Checking formatting...")
          const fmtResult = await $`cd ${directory} && cargo fmt --all -- --check`
          if (fmtResult.exitCode !== 0) {
            throw new Error("Formatting check failed! Run: cargo fmt --all")
          }
          console.log("✅ Formatting OK")
          
          // 2. Clippy check
          console.log("🔍 Running clippy...")
          const clippyResult = await $`cd ${directory} && cargo clippy --all -- -D warnings`
          if (clippyResult.exitCode !== 0) {
            throw new Error("Clippy check failed! Fix warnings before committing.")
          }
          console.log("✅ Clippy OK")
          
          // 3. Compilation check
          console.log("🔧 Checking compilation...")
          const checkResult = await $`cd ${directory} && cargo check --all`
          if (checkResult.exitCode !== 0) {
            throw new Error("Compilation check failed!")
          }
          console.log("✅ Compilation OK")
          
          // 4. Test check (hybrid module as minimum)
          console.log("🧪 Running hybrid module tests...")
          const testResult = await $`cd ${directory} && cargo test hybrid:: -- --test-threads=1`
          if (testResult.exitCode !== 0) {
            throw new Error("Tests failed! Fix tests before committing.")
          }
          console.log("✅ Tests OK")
          
          console.log("🎉 ALL CI CHECKS PASSED! Proceeding with git operation...")
          
        } catch (error) {
          console.error("❌ CI CHECKS FAILED!")
          console.error(error.message)
          console.error("Git operation blocked. Fix issues and try again.")
          throw new Error(`CI checks failed: ${error.message}`)
        }
      }
      
      // Allow the tool to proceed
      return output
    },
    
    // Also provide a manual check command
    tool: {
      checkci: {
        description: "Run CI checks manually",
        args: {},
        async execute(args, ctx) {
          console.log("🧪 Running manual CI checks...")
          
          try {
            // 1. Formatting
            console.log("📝 Checking formatting...")
            await ctx.$`cd ${ctx.directory} && cargo fmt --all -- --check`
            console.log("✅ Formatting OK")
            
            // 2. Clippy
            console.log("🔍 Running clippy...")
            await ctx.$`cd ${ctx.directory} && cargo clippy --all -- -D warnings`
            console.log("✅ Clippy OK")
            
            // 3. Compilation
            console.log("🔧 Checking compilation...")
            await ctx.$`cd ${ctx.directory} && cargo check --all`
            console.log("✅ Compilation OK")
            
            // 4. Tests
            console.log("🧪 Running tests...")
            await ctx.$`cd ${ctx.directory} && cargo test hybrid:: -- --test-threads=1`
            console.log("✅ Tests OK")
            
            return "🎉 ALL CI CHECKS PASSED! Ready to commit."
            
          } catch (error) {
            return `❌ CI CHECKS FAILED: ${error.message}`
          }
        }
      }
    }
  }
}