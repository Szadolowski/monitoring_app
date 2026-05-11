using System;
using System.Windows;
using System.Windows.Controls;

namespace FileMonitorAgent.App.Views
{
    public partial class GroupSelectionView : UserControl
    {
        // To jest nasz "posłaniec" – dzięki niemu MainWindow dowie się, co kliknięto
        public event Action<int>? OnGroupSelected;

        public GroupSelectionView()
        {
            InitializeComponent();
        }

        private void Group_Click(object sender, RoutedEventArgs e)
        {
            // Sprawdzamy, który przycisk został kliknięty i wyciągamy jego Tag (ID grupy)
            if (sender is Button button && button.Tag != null)
            {
                if (int.TryParse(button.Tag.ToString(), out int groupId))
                {
                    // Wywołujemy zdarzenie, na które "nasłuchuje" MainWindow
                    OnGroupSelected?.Invoke(groupId);
                }
            }
        }
    }
}
