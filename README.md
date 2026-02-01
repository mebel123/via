# VIA Desktop Recorder

VIA is a cross platform desktop application built with Tauri and Next.js.  
It demonstrates a global system shortcut driven recording workflow with a clear separation between UI logic and native desktop capabilities.

The focus of the project is a clean architecture, reproducible setup, and a foundation that can be extended with speech to text and AI based enrichment.

## Motivation

Many modern applications require system wide interactions that work independently of window focus.  
Purely browser based solutions quickly reach their limits in these scenarios.

VIA shows how a modern web frontend can be combined with native desktop features without sacrificing developer experience or performance.

## Core Features

1. Global system shortcut independent of the active window  
2. Toggle mechanism to start and stop a recording  
3. Clear visual recording state in the UI  
4. Separation of frontend logic and native desktop integration  
5. Hot reload capable development workflow

## Technology Stack

1. Tauri for the desktop runtime and system APIs  
2. Next.js for the user interface  
3. React with client components  
4. TypeScript  
5. Rust in the Tauri backend  
6. Node.js version 20 or higher

## Architecture Overview

The application is split into two clearly separated parts.

Frontend  
The frontend runs as a Next.js application and is responsible for UI, status visualization, and user interaction.

Backend  
The Tauri backend provides access to native desktop capabilities such as global shortcuts and, in later stages, audio and recording APIs.

Communication between both layers is handled via Tauri plugins and events.

## Global Shortcut Flow

1. The application starts and registers a global shortcut  
2. The shortcut is captured system wide  
3. On the first trigger, recording is started  
4. On the next trigger, recording is stopped  
5. The current state is reflected immediately in the UI

## Development

Requirements:

1. Node.js version 20 or newer  
2. Rust toolchain  
3. Tauri CLI

Setup:

1. Clone the repository  
2. Change into the app directory  
3. Install dependencies


```bash
npm install
```


```bash
npm install @tauri-apps/plugin-global-shortcut
npx tauri add global-shortcut
```

Start the development environment:

```bash
npx tauri dev
```

The Next.js development server is started automatically and embedded into the Tauri desktop window.

## Project Structure

app  
Contains the Next.js application and all UI related logic

src tauri  
Contains the Tauri configuration, Rust code, and desktop integration

## Current Status

Implemented:

1. Stable global shortcut registration  
2. Toggle logic for recording state  
3. Visual recording indicator in the UI  
4. Solid base architecture for further extensions

Planned:

1. Audio recording  
2. Speech to text integration  
3. Structured output and export  
4. AI assisted post processing

## Contest Relevance

This project demonstrates:

1. A clean combination of web technologies and native desktop APIs  
2. Understanding of system level interactions and UX constraints  
3. Scalable architecture instead of throwaway prototype code  
4. Focus on real world product requirements

## License

MIT