using System;
using System.Collections.Generic;
using System.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input; // Wymagane dla KeyEventArgs
using FileMonitorAgent.App.Models;
using FileMonitorAgent.App.Services;

namespace FileMonitorAgent.App.Views
{
    public partial class ProfileSelectionView : UserControl
    {
        private readonly int _groupId;
        private readonly AuthService _authService;
        private IEnumerable<MonitoringProfile> _allProfiles;

        public event Action<IEnumerable<MonitoringProfile>>? OnMonitoringStarted;
        public event Action? OnBackRequested;

        public ProfileSelectionView(
            int groupId,
            IEnumerable<MonitoringProfile> profiles,
            AuthService authService
        )
        {
            InitializeComponent();
            _groupId = groupId;
            _authService = authService;
            _allProfiles = profiles;

            // Podpinamy dane pod obie listy
            ProfilesCheckList.ItemsSource = _allProfiles;
            ProfilesList.ItemsSource = _allProfiles;

            SetupView();
        }

        private void SetupView()
        {
            if (_groupId == 1) // BIURO
            {
                TitleText.Text = "Autoryzacja Biura";
                OfficePasswordSection.Visibility = Visibility.Visible;
                // Skupienie od razu na polu hasła
                OfficePassword.Focus();
            }
            else // BRYGADZISTA
            {
                TitleText.Text = "Wybór Budowy";
                ProfilesList.Visibility = Visibility.Visible;
                BtnStart.Visibility = Visibility.Visible;
                ForemanPasswordSection.Visibility = Visibility.Visible;
            }
        }

        // Obsługa Entera
        private void OfficePassword_KeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key == Key.Enter)
                BtnUnlock_Click(sender, e);
        }

        private void ForemanPassword_KeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key == Key.Enter)
                BtnStart_Click(sender, e);
        }

        private async void BtnUnlock_Click(object sender, RoutedEventArgs e)
        {
            bool isValid = await _authService.VerifyGroupPasswordAsync(
                _groupId,
                OfficePassword.Password
            );
            if (isValid)
            {
                OfficePasswordSection.Visibility = Visibility.Collapsed;
                ProfilesCheckList.Visibility = Visibility.Visible; // Pokazujemy checkboxy
                BtnStart.Visibility = Visibility.Visible;
                StatusLabel.Text = "";
            }
            else
            {
                StatusLabel.Text = "Błędne hasło grupy!";
            }
        }

        private void BtnStart_Click(object sender, RoutedEventArgs e)
        {
            List<MonitoringProfile> selected;

            if (_groupId == 1) // Biuro z CheckBoxów
            {
                selected = _allProfiles.Where(p => p.IsSelected).ToList();
            }
            else // Brygadzista ze zwykłej listy
            {
                selected = ProfilesList.SelectedItems.Cast<MonitoringProfile>().ToList();

                if (selected.Any())
                {
                    var profile = selected.First();
                    if (
                        string.IsNullOrEmpty(ForemanPassword.Password)
                        || !BCrypt.Net.BCrypt.Verify(ForemanPassword.Password, profile.PasswordHash)
                    )
                    {
                        StatusLabel.Text = "Błędne hasło budowy!";
                        return;
                    }
                }
            }

            if (!selected.Any())
            {
                StatusLabel.Text = "Wybierz przynajmniej jeden element!";
                return;
            }

            OnMonitoringStarted?.Invoke(selected);
        }

        private void BtnBack_Click(object sender, RoutedEventArgs e) => OnBackRequested?.Invoke();

        public void ShowError(string message) => StatusLabel.Text = message;
    }
}
