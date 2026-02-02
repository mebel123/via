# VIA Desktop Recorder

VIA is a desktop application that records voice input, transcribes it, and enriches it through AI-powered processing. The goal is to seamlessly transform spoken thoughts into structured information—whether as a note, formatted text, or context-aware output.

The application was developed as part of a competition where Next.js was a required foundation.

## Features

*   **Global Hotkey:** The app can be activated at any time via a keyboard shortcut (Default: `Cmd+Shift+Space` or `Ctrl+Shift+Space`), regardless of the active window.
*   **Voice Pipeline:**
    1.  **Recording:** Audio is recorded locally.
    2.  **Transcription:** Speech-to-text conversion (via OpenAI Whisper).
    3.  **Enrichment:** Extraction of entities (persons, organizations, projects) from the transcript.
*   **Modern UI:** Next.js frontend embedded in Tauri.

## Tech Stack

*   **Frontend:** Next.js (React, TypeScript)
*   **Desktop Runtime:** Tauri (Rust)
*   **Backend Logic:** Rust (for audio recording, pipeline orchestration, API calls)
*   **AI Integration:** OpenAI API (Whisper for transcription, GPT for entity extraction)

## Architecture

The application follows a clear separation between Frontend (UI) and Backend (System Logic):

*   **Frontend (Next.js):** Visualizes the status (Ready, Recording, Processing) and displays results. Communicates with the backend via Tauri Commands and Events.
*   **Backend (Rust):**
    *   Manages the global shortcut.
    *   Controls audio recording.
    *   Executes the "Processing Pipeline": A sequence of steps (Transcription -> Entity Extraction) processed sequentially.

### Folder Structure

```
via/
├── app/                              # Next.js Frontend & Tauri Root
│   ├── src/                          # React Components & Pages
│   ├── src-tauri/                    # Rust Backend & Tauri Config
│   │   ├── src/
│   │   │   ├── agents/               # Ai LLM Agents
│   │   │   ├── commands/             # Backend logic from ui
│   │   │   ├── resolvers/            # Logic resolvers
│   │   │   │   ├── steps/            # Resolver steps
│   │   │   ├── pipeline/             # Logic for the processing pipeline
│   │   │   │   ├── transcription.rs  # Whisper API Integration
│   │   │   │   ├── entities.rs       # Entity Extraction
│   │   │   │   └── ...
│   │   │   ├── store/                # Backend Stores
│   │   │   ├── lib.rs                # Tauri Setup & Commands
│   │   │   └── main.rs               # Entry Point
│   │   ├── capabilities/             # Tauri Permissions
│   │   └── tauri.conf.json           # Tauri Configuration
│   ├── package.json                  # Frontend Dependencies
│   └── ...
└── README.md
```

## Setup & Installation

Prerequisites:
*   Node.js (v20+ recommended, see `.nvmrc` in `app/`)
*   Rust Toolchain (via `rustup`)
*   Tauri CLI (installed locally)

### 1. Clone Repository

```bash
git clone <repo-url>
cd via
```

### 2. Install Dependencies

Switch to the `app` directory, as it contains both the frontend and the Tauri configuration.

```bash
cd app
npm install
```

*Note: The necessary Tauri plugins and the CLI are already defined in `package.json` and will be installed automatically.*

### 3. Configure Environment Variables

Create a `.env` file in the `app` directory to store your OpenAI API Key. This is required for transcription and enrichment.

```bash
# app/.env
OPENAI_API_KEY=sk-proj-....
```

### 4. Start Development Environment

Start the app in development mode. This launches the Next.js server and opens the Tauri window.

```bash
npx tauri dev
```

## Usage

1.  Start the app.
2.  Press the global hotkey (configured in `app/src-tauri/tauri.conf.json` or hardcoded in the Rust backend, e.g., `Cmd+Shift+Space`).
3.  The recording window appears (or the status changes). Speak your note.
4.  Press the hotkey again to stop recording.
5.  Processing starts automatically (Transcription -> Extraction).
6.  The result is displayed in the frontend or saved to the file system (see `app/src-tauri/src/pipeline/`).

## Design Decisions

*   **Tauri over Electron:** Lower resource usage and higher security through Rust.
*   **Pipeline Pattern:** Processing steps (Transcription, Enrichment) are modular (`PipelineStep` Trait), allowing for easy addition of new steps (e.g., Summarization, To-Do Detection).
*   **Next.js:** Using a modern web framework enables rapid UI iterations and access to a vast ecosystem of libraries.

    1.	Memo / Aufnahme
    2.	Entities
    3.	Evidence
    4.	Knowledge
    5.	Überarbeitung / Resolver / Fragen
          Transcription
          → EntityExtraction
          → PersonRelationAgent
          → ContextRelationAgent
          → EvidenceAggregator
          → KnowledgeBuilder
          → (später) Resolver
