using System;

namespace FileMonitorAgent.App.Models
{
    public class LogEntry
    {
        public string Time { get; set; } = DateTime.Now.ToString("HH:mm:ss");
        public string Message { get; set; } = string.Empty;
        public string Icon { get; set; } = "📄";
        public string FolderPath { get; set; } = string.Empty;
    }
}
