using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Windows;
using System.Windows.Controls;
using FileMonitorAgent.App.Models;
using FileMonitorAgent.App.Services;

namespace FileMonitorAgent.App.Views
{
    public partial class MainDashboardView : UserControl
    {
        private readonly FileMonitorService _monitorService;

        // ObservableCollection automatycznie odświeża listę w UI, gdy dodajemy elementy
        public ObservableCollection<LogEntry> LogEntries { get; } = new();

        public event Action? OnLogoutRequested;

        public MainDashboardView(MonitoringProfile profile, IEnumerable<WatchedPath> paths)
        {
            InitializeComponent();
            _monitorService = new FileMonitorService();

            // Podpięcie danych do list w XAML
            LogsList.ItemsSource = LogEntries;
            WatchedFoldersList.ItemsSource = paths; // To zasila nasz nowy Expander

            // Statystyki w nagłówku i stopce
            ProfileNameText.Text = $"Profil: {profile.ProfileName}";
            if (paths.Any())
            {
                // Jeśli jest wiele profilów (Biuro), PathsCountText jest w StatusBarze
                PathsCountText.Text = $"Monitorowane foldery: {paths.Count()}";
            }

            // Obsługa zdarzeń z serwisu (teraz z dwoma parametrami: wiadomość i ścieżka)
            _monitorService.OnFileChanged += (msg, folderPath) =>
            {
                // FileSystemWatcher działa na innym wątku, więc musimy "wejść" do wątku UI
                Dispatcher.Invoke(() =>
                {
                    LogEntries.Insert(
                        0,
                        new LogEntry
                        {
                            Message = msg,
                            FolderPath = folderPath, // Przechowujemy, żeby przycisk wiedział co otworzyć
                            Icon = msg.Contains("[NOWY]") ? "🆕" : "📝",
                        }
                    );
                });
            };

            _monitorService.OnFileChanged += (msg, folderPath) =>
            {
                Dispatcher.Invoke(() =>
                {
                    // Dodajemy do listy w UI (to już mamy)
                    LogEntries.Insert(
                        0,
                        new LogEntry
                        {
                            Message = msg,
                            FolderPath = folderPath,
                            Icon = msg.Contains("[NOWY]") ? "🆕" : "📝",
                        }
                    );

                    // NOWE: Wysyłamy powiadomienie systemowe Windows
                    NotificationService.ShowFileNotification("Nowy plik wykryty!", msg, folderPath);
                });
            };

            _monitorService.StartMonitoring(paths);
            LogEntries.Add(
                new LogEntry { Message = "System monitorowania uruchomiony.", Icon = "🚀" }
            );
        }

        /// <summary>
        /// Logika przycisku "Otwórz folder" przy każdym logu
        /// </summary>
        private void BtnOpenFolder_Click(object sender, RoutedEventArgs e)
        {
            // Wyciągamy ścieżkę zapisaną w Tagu przycisku (ustawiliśmy to w XAML)
            if (
                sender is Button btn
                && btn.Tag is string folderPath
                && !string.IsNullOrEmpty(folderPath)
            )
            {
                try
                {
                    // Uruchamiamy proces eksploratora Windows
                    System.Diagnostics.Process.Start("explorer.exe", folderPath);
                }
                catch (Exception ex)
                {
                    MessageBox.Show(
                        $"Nie można otworzyć folderu. Prawdopodobnie ścieżka jest nieosiągalna.\nBłąd: {ex.Message}",
                        "Błąd Eksploratora"
                    );
                }
            }
        }

        private void BtnLogout_Click(object sender, RoutedEventArgs e)
        {
            // ZATRZYMUJEMY watchery przed wyjściem - to bardzo ważne dla pamięci RAM
            _monitorService.StopAll();
            OnLogoutRequested?.Invoke();
        }
    }
}
