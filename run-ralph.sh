#!/bin/bash
# Run Ralph from the Driftlands project root
# Usage: ./run-ralph.sh [max_iterations]

set -e

MAX_ITERATIONS=${1:-40}
RALPH_DIR="$(pwd)/ralph"
PRD_FILE="$RALPH_DIR/prd.json"
PROGRESS_FILE="$RALPH_DIR/progress.txt"
CLAUDE_MD="$RALPH_DIR/CLAUDE.md"

if [ ! -f "$PRD_FILE" ]; then
  echo "Error: No ralph/prd.json found in current directory"
  exit 1
fi

echo "Starting Ralph - Tool: claude - Max iterations: $MAX_ITERATIONS"
echo "PRD: $PRD_FILE"
echo "Progress: $PROGRESS_FILE"

for i in $(seq 1 $MAX_ITERATIONS); do
  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS"
  echo "==============================================================="

  # Check if all stories pass
  ALL_PASS=$(jq '[.userStories[] | .passes] | all' "$PRD_FILE" 2>/dev/null || echo "false")
  if [ "$ALL_PASS" = "true" ]; then
    echo ""
    echo "All user stories are passing! Ralph is done."
    exit 0
  fi

  # Show next story
  NEXT_STORY=$(jq -r '[.userStories[] | select(.passes == false)] | sort_by(.priority) | .[0] | "\(.id) - \(.title)"' "$PRD_FILE" 2>/dev/null || echo "unknown")
  echo "Next story: $NEXT_STORY"
  echo ""

  # Run claude with the ralph prompt
  OUTPUT=$(claude --dangerously-skip-permissions -p "$(cat "$CLAUDE_MD")" 2>&1 | tee /dev/stderr) || true

  # Check for completion signal
  if echo "$OUTPUT" | grep -q "COMPLETE"; then
    echo ""
    echo "Ralph completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    exit 0
  fi

  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check ralph/progress.txt for status."
exit 1
