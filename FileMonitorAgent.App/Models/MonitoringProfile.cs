namespace FileMonitorAgent.App.Models
{
    public class MonitoringProfile
    {
        public int Id { get; set; }
        public int GroupId { get; set; }
        public string ProfileName { get; set; } = string.Empty;
        public string PasswordHash { get; set; } = string.Empty;
        public bool IsSelected { get; set; }
    }
}
