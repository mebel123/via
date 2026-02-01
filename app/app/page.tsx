"use client";
import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";

export default function Home() {
    const [logs, setLogs] = useState<string[]>([]);
    const [status, setStatus] = useState<string>("Initialisiere...");
    const [isRecording, setIsRecording] = useState(false);

    const SHORTCUT = "CommandOrControl+Shift+U";

    useEffect(() => {
        document.title = isRecording ? "Lucida ● REC" : "Lucida";
    }, [isRecording]);

    useEffect(() => {
        let isMounted = true;

        async function setupShortcut() {
            try {
                await unregisterAll();


                await register(SHORTCUT, (event) => {
                    if (event.state !== "Pressed") return;

                    setIsRecording((prev) => {
                        const nextState = !prev;
                        const timestamp = new Date().toLocaleTimeString();
                        if (nextState) {
                            invoke("start_recording").catch((e) => {
                                console.warn("start_recording failed:", e);
                            });
                        } else {
                            invoke("stop_recording").catch((e) => {
                                console.warn("stop_recording failed:", e);
                            });
                        }
                        const message = nextState
                            ? `● Aufnahme gestartet um ${timestamp}`
                            : `■ Aufnahme beendet um ${timestamp}`;

                        setLogs((prevLogs) => [message, ...prevLogs]);
                        setStatus(nextState ? "Aufnahme läuft…" : `Bereit! Drücke ${SHORTCUT}`);

                        return nextState;
                    });
                });

                if (isMounted) {
                    setStatus(`Bereit! Drücke ${SHORTCUT}`);
                }
            } catch (err) {
                console.error("Hotkey Fehler:", err);
                if (isMounted) setStatus("Fehler beim Registrieren (siehe Konsole)");
            }
        }

        setupShortcut();

        // WICHTIG: Cleanup-Funktion
        // Wird ausgeführt, wenn die Komponente entladen wird (oder beim Hot-Reload)
        return () => {
            isMounted = false;
            unregisterAll(); // Löscht den Shortcut sauber im Backend
        };
    }, []);

    return (
        <main className="flex min-h-screen flex-col items-center justify-center p-24 bg-gray-900 text-white">
            <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm lg:flex flex-col gap-8">

                <h1 className="text-4xl font-bold text-blue-400">Tauri Hotkey Demo</h1>

                {/* Status Anzeige */}
                <div className="p-4 border border-gray-700 rounded-lg bg-gray-800 w-full text-center">
                    <p className="text-xl">{status}</p>
                    <p className="text-gray-400 text-sm mt-2">
                        Minimiere dieses Fenster und drücke die Tasten, um zu testen.
                    </p>
                </div>

                {/* Recording Indicator */}
                <div className="flex items-center gap-3 justify-center">
                    <div
                        className={`w-4 h-4 rounded-full transition-colors duration-300 ${
                            isRecording ? "bg-red-500 animate-pulse shadow-[0_0_10px_red]" : "bg-gray-600"
                        }`}
                    />
                    <span className={`text-lg font-bold ${isRecording ? "text-red-400" : "text-gray-500"}`}>
                        {isRecording ? "REC" : "IDLE"}
                    </span>
                </div>

                {/* Log Bereich */}
                <div className="w-full h-64 overflow-y-auto border border-gray-700 rounded-lg bg-black p-4 shadow-inner">
                    {logs.length === 0 ? (
                        <p className="text-gray-500 italic text-center mt-10">Warte auf Eingabe...</p>
                    ) : (
                        logs.map((log, index) => (
                            <div key={index} className="mb-2 border-b border-gray-800 pb-1 text-green-400 font-mono">
                                {log}
                            </div>
                        ))
                    )}
                </div>

                <button
                    onClick={() => setLogs([])}
                    className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition text-white border border-gray-500"
                >
                    Logs leeren
                </button>

            </div>
        </main>
    );
}