using System;
using System.Collections.Generic;
using System.Linq; // KLUCZOWE: Obsługa Select() i First()
using System.Threading.Tasks;
using System.Windows;
using FileMonitorAgent.App.Database;
using FileMonitorAgent.App.Models;
using FileMonitorAgent.App.Services;
using FileMonitorAgent.App.Views;

namespace FileMonitorAgent.App
{
    public partial class MainWindow : Window
    {
        private readonly AuthService _authService;

        public MainWindow()
        {
            InitializeComponent();

            // Inicjalizacja usług
            var dbFactory = new DbConnectionFactory();
            _authService = new AuthService(dbFactory);

            // Start aplikacji
            ShowGroupSelection();
        }

        public void ShowGroupSelection()
        {
            var view = new GroupSelectionView();
            view.OnGroupSelected += async (groupId) => await ShowProfileSelection(groupId);
            MainContent.Content = view;
        }

        private async Task ShowProfileSelection(int groupId)
        {
            try
            {
                var profiles = await _authService.GetProfilesByGroupAsync(groupId);
                var profileView = new ProfileSelectionView(groupId, profiles, _authService);

                profileView.OnBackRequested += () => ShowGroupSelection();

                profileView.OnMonitoringStarted += async (selectedProfiles) =>
                {
                    // Wyciągamy ID zaznaczonych profili (działa dzięki System.Linq)
                    var profileIds = selectedProfiles.Select(p => p.Id);

                    // Pobieramy ścieżki z bazy dla wszystkich wybranych profili
                    var paths = await _authService.GetWatchedPathsForProfilesAsync(profileIds);

                    // Przekazujemy pierwszy z wybranych profili do dashboardu (dla nazwy)
                    ShowDashboard(selectedProfiles.First(), paths);
                };

                MainContent.Content = profileView;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Błąd podczas ładowania profili: {ex.Message}", "Błąd Bazy");
            }
        }

        private void ShowDashboard(MonitoringProfile profile, IEnumerable<WatchedPath> paths)
        {
            var dashboard = new MainDashboardView(profile, paths);

            // Powrót do menu głównego przy wylogowaniu
            dashboard.OnLogoutRequested += () => ShowGroupSelection();

            MainContent.Content = dashboard;
        }
    }
}
