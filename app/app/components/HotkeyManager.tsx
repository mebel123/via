"use client"; // Wichtig für Next.js App Router

import { useEffect } from 'react';
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut';

export default function HotkeyManager() {

    useEffect(() => {
        async function setupHotkeys() {
            try {
                // Vorherige Shortcuts bereinigen (Good Practice, um Doppel-Registrierung im Dev-Mode zu vermeiden)
                await unregisterAll();

                // Beispiel 1: Einfacher Shortcut
                await register('CommandOrControl+Shift+C', (event) => {
                    console.log('Shortcut ausgelöst:', event);
                    if (event.state === "Pressed") {
                        alert("Cmd+Shift+C wurde gedrückt!");
                    }
                });

                console.log('Globaler Shortcut registriert');
            } catch (error) {
                console.error('Fehler beim Registrieren des Shortcuts:', error);
            }
        }

        setupHotkeys();

        // Cleanup beim Unmounten der Komponente
        return () => {
            // Optional: unregisterAll() hier aufrufen, wenn die Shortcuts nur leben sollen,
            // solange diese Komponente existiert.
        };
    }, []);

    return (
        <div className="p-4 border rounded">
            <p>Drücke <b>Strg+Shift+C</b> (oder Cmd+Shift+C auf Mac), um den Global Shortcut zu testen.</p>
        </div>
    );
}