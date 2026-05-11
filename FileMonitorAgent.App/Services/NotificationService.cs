using Microsoft.Toolkit.Uwp.Notifications;

namespace FileMonitorAgent.App.Services
{
    public static class NotificationService
    {
        public static void ShowFileNotification(string title, string message, string folderPath)
        {
            // Budujemy powiadomienie
            var toast = new ToastContentBuilder()
                .AddText(title)
                .AddText(message)
                .AddArgument("folderPath", folderPath);

            // W WPF używamy metody Show() bezpośrednio,
            // ale po zmianie TargetFramework w kroku 1 powinna być już widoczna.
            toast.Show();
        }
    }
}
