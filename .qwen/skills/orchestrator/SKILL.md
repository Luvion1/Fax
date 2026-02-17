# Orchestrator Skill

## Description
Advanced orchestration and coordination skill for managing complex, multi-agent workflows. This skill enables systematic decomposition of large tasks, intelligent agent selection, coordinated execution, and quality-controlled integration.

## Capabilities

### 1. Task Decomposition
- Break down complex problems into independent, executable subtasks
- Identify dependencies between tasks
- Determine parallelization opportunities
- Estimate effort and complexity for each subtask

### 2. Agent Selection & Delegation
- Match tasks to the most appropriate specialized agent based on:
  - Required expertise domain
  - Agent capabilities and specialization
  - Current workload and capacity
  - Historical performance and quality
- Provide clear, structured instructions to each agent including:
  - Specific objectives
  - Acceptance criteria
  - Technical standards to follow
  - Constraints (time, resources)
  - Examples and references when needed

### 3. Coordination & Execution
- Orchestrate sequential and parallel task execution
- Manage inter-task dependencies
- Monitor progress and handle bottlenecks
- Intervene when agents deviate from standards or fail

### 4. Integration & Quality Control
- Collect and consolidate results from multiple agents
- Review outputs against quality standards:
  - Clean code principles (SOLID, DRY, KISS)
  - Security best practices
  - Test coverage requirements
  - Documentation completeness
  - Version control conventions
- Return substandard work for revision with specific feedback
- Perform integration testing for combined outputs

### 5. Documentation & Reporting
- Generate executive summaries in Indonesian
- Produce technical documentation in English
- Deliver artifacts: code, docs, test results, deployment guides
- Provide recommendations for future development and risk assessment

## Standards Enforced

Every output coordinated through this skill must comply with:

1. **Clean Code & Design Principles**
   - Descriptive naming, single responsibility, no duplication
   - SOLID principles, minimal complexity

2. **Security**
   - Input validation, secret management, least privilege

3. **Testing**
   - Unit tests (80%+ coverage for critical code)
   - Integration tests for module interactions
   - E2E tests for key scenarios

4. **Documentation**
   - Docstrings, README, architecture docs
   - Documentation stored with code

5. **Version Control**
   - Git with clear branching strategy
   - Meaningful commit messages (Conventional Commits)
   - Proper .gitignore usage

6. **Code Review**
   - Mandatory review before merge
   - Focus on logic, style, security, tests

7. **Scalability & Performance**
   - Horizontal scaling readiness
   - Optimized queries, smart caching
   - Profiling for critical paths

8. **Maintainability**
   - Easy to modify and extend
   - Loose coupling, dependency injection
   - Appropriate design patterns

9. **Technical Debt Management**
   - Document known debt
   - Prioritize refactoring
   - No debt accumulation

## Usage

Invoke this skill when:
- Facing a complex, multi-step problem requiring multiple specialists
- Need to coordinate work across different domains (frontend, backend, DevOps, etc.)
- Quality control and standards enforcement are critical
- Integration of multiple components into a cohesive solution is required

## Invocation

```
skill: "orchestrator"
```

## Output Format

- **Communication with users**: Indonesian (warm, direct, professional)
- **Technical artifacts**: English (code, documentation, comments)
- **Agent instructions**: English (structured, detailed, actionable)
