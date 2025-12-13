## ADDED Requirements

### Requirement: Long-Running Agent Harness
The system SHALL provide a harness for agents to work across multiple context windows with progress preservation and clean state management.

#### Scenario: Initializer agent setup
- **WHEN** a new project is started with a high-level prompt
- **THEN** an initializer agent SHALL create a feature list file
- **AND** initialize a git repository with progress tracking
- **AND** create environment setup scripts
- **AND** establish a claude-progress.txt file for session tracking

#### Scenario: Coding agent incremental progress
- **WHEN** a coding agent starts a new session
- **THEN** it SHALL read the progress file and git history
- **AND** choose a single feature to implement
- **AND** leave the codebase in a merge-ready state
- **AND** commit changes with descriptive messages
- **AND** update the progress file before ending

#### Scenario: Session recovery
- **WHEN** an agent encounters a broken state from previous session
- **THEN** it SHALL use git to revert to last working state
- **AND** diagnose and fix the breaking changes
- **AND** continue with feature implementation
- **AND** document the recovery process in progress file

#### Scenario: Context window management
- **WHEN** an agent approaches context window limits
- **THEN** it SHALL compress and summarize completed work
- **AND** store essential information in progress file
- **AND** create a clean handoff for next session
- **AND** ensure no critical information is lost

### Requirement: Testing Integration
The system SHALL integrate comprehensive testing capabilities for autonomous verification of feature completion.

#### Scenario: End-to-end testing
- **WHEN** an agent implements a feature
- **THEN** it SHALL use browser automation to test the feature
- **AND** capture screenshots for visual verification
- **AND** verify all user interactions work correctly
- **AND** only mark feature as complete after successful testing

#### Scenario: Self-verification
- **WHEN** an agent completes a feature implementation
- **THEN** it SHALL run all relevant tests automatically
- **AND** verify the feature works as specified
- **AND** check for regression in existing functionality
- **AND** document test results in progress file

#### Scenario: Bug detection and reporting
- **WHEN** testing reveals a bug
- **THEN** the agent SHALL create a detailed bug report
- **AND** attempt to fix the bug immediately
- **AND** if unable to fix, escalate with clear reproduction steps
- **AND** update feature status accordingly

### Requirement: Memory Compression
The system SHALL efficiently manage memory across agent sessions to preserve context and progress.

#### Scenario: Progress file management
- **WHEN** an agent completes work on a feature
- **THEN** it SHALL update the progress file with concise summary
- **AND** include git commit references
- **AND** note any outstanding issues or TODOs
- **AND** update feature status (passing/failing)

#### Scenario: Session summarization
- **WHEN** an agent ends a session
- **THEN** it SHALL create a session summary
- **AND** compress completed work into key points
- **AND** identify next steps for continuation
- **AND** ensure summary fits within context window limits

#### Scenario: Information retrieval
- **WHEN** an agent starts a new session
- **THEN** it SHALL quickly reconstruct project state
- **AND** understand what has been completed
- **AND** identify current priorities
- **AND** resume work with minimal overhead