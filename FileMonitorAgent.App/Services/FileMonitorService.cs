using System;
using System.Collections.Concurrent;
using System.IO;
using FileMonitorAgent.App.Models;

namespace FileMonitorAgent.App.Services
{
    public class FileMonitorService
    {
        // Trzymamy tu wszystkie aktywne watchery, żeby móc je potem zatrzymać
        private readonly ConcurrentDictionary<long, FileSystemWatcher> _watchers = new();

        // Zdarzenie, które pośle info do UI
        public event Action<string, string>? OnFileChanged;

        public void StartMonitoring(IEnumerable<WatchedPath> paths)
        {
            foreach (var wp in paths)
            {
                try
                {
                    if (!Directory.Exists(wp.Path))
                        continue;

                    var watcher = new FileSystemWatcher(wp.Path)
                    {
                        NotifyFilter =
                            NotifyFilters.FileName
                            | NotifyFilters.LastWrite
                            | NotifyFilters.CreationTime,
                        Filter = "*.*",
                        EnableRaisingEvents = true,
                        InternalBufferSize = 65536,
                    };

                    watcher.Created += (s, e) =>
                    {
                        string dir = Path.GetDirectoryName(e.FullPath) ?? wp.Path;
                        OnFileChanged?.Invoke($"[NOWY] {wp.NotificationMsg}: {e.Name}", dir);
                    };

                    watcher.Changed += (s, e) =>
                    {
                        string dir = Path.GetDirectoryName(e.FullPath) ?? wp.Path;
                        OnFileChanged?.Invoke($"[ZMIANA] {wp.NotificationMsg}: {e.Name}", dir);
                    };

                    // POPRAWKA TUTAJ: Dodajemy pusty string jako drugi argument (ścieżkę)
                    watcher.Error += (s, e) =>
                    {
                        OnFileChanged?.Invoke(
                            $"! BŁĄD BUFORA: {wp.Path}. Restartowanie...",
                            wp.Path
                        );
                        RestartWatcher(wp);
                    };

                    _watchers.TryAdd(wp.Id, watcher);
                }
                catch (Exception ex)
                {
                    // POPRAWKA TUTAJ: Dodajemy wp.Path jako drugi argument
                    OnFileChanged?.Invoke($"! BŁĄD STARTU {wp.Path}: {ex.Message}", wp.Path);
                }
            }
        }

        private void RestartWatcher(WatchedPath wp)
        {
            if (_watchers.TryRemove(wp.Id, out var oldWatcher))
            {
                oldWatcher.Dispose();
                StartMonitoring(new[] { wp });
            }
        }

        public void StopAll()
        {
            foreach (var watcher in _watchers.Values)
            {
                watcher.EnableRaisingEvents = false;
                watcher.Dispose();
            }
            _watchers.Clear();
        }
    }
}
