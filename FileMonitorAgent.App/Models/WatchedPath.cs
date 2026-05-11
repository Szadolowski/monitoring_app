namespace FileMonitorAgent.App.Models
{
    public class WatchedPath
    {
        public long Id { get; set; }
        public int ProfileId { get; set; }
        public string Path { get; set; } = string.Empty;
        public string? NotificationMsg { get; set; }
        public bool IsActive { get; set; }
    }
}