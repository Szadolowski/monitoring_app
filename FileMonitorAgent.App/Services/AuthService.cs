using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Dapper;
using FileMonitorAgent.App.Database;
using FileMonitorAgent.App.Models;

namespace FileMonitorAgent.App.Services
{
    public class AuthService
    {
        private readonly DbConnectionFactory _dbFactory;

        public AuthService(DbConnectionFactory dbFactory)
        {
            _dbFactory = dbFactory;
        }

        public async Task<IEnumerable<MonitoringProfile>> GetProfilesByGroupAsync(int groupId)
        {
            using var connection = await _dbFactory.CreateConnectionAsync();
            const string sql =
                @"
                SELECT id, group_id as GroupId, profile_name as ProfileName, password_hash as PasswordHash 
                FROM public.monitoring_profiles 
                WHERE group_id = @GroupId";
            return await connection.QueryAsync<MonitoringProfile>(sql, new { GroupId = groupId });
        }

        // NOWE: Pobieranie ścieżek dla wielu profili naraz (dla Biura)
        public async Task<IEnumerable<WatchedPath>> GetWatchedPathsForProfilesAsync(
            IEnumerable<int> profileIds
        )
        {
            using var connection = await _dbFactory.CreateConnectionAsync();
            const string sql =
                @"
                SELECT id, profile_id as ProfileId, path, notification_msg as NotificationMsg, is_active as IsActive 
                FROM public.watched_paths 
                WHERE profile_id = ANY(@ProfileIds) AND is_active = true";

            return await connection.QueryAsync<WatchedPath>(
                sql,
                new { ProfileIds = profileIds.ToArray() }
            );
        }

        // NOWE: Sprawdzenie hasła dla grupy (używamy hasła pierwszego profilu w grupie)
        public async Task<bool> VerifyGroupPasswordAsync(int groupId, string password)
        {
            using var connection = await _dbFactory.CreateConnectionAsync();
            const string sql =
                "SELECT password_hash FROM public.monitoring_profiles WHERE group_id = @GroupId LIMIT 1";
            var hash = await connection.QueryFirstOrDefaultAsync<string>(
                sql,
                new { GroupId = groupId }
            );

            if (string.IsNullOrEmpty(hash))
                return false;
            return BCrypt.Net.BCrypt.Verify(password, hash);
        }

        public async Task<MonitoringProfile?> AuthenticateAsync(
            MonitoringProfile selectedProfile,
            string? password
        )
        {
            if (
                string.IsNullOrEmpty(password) || string.IsNullOrEmpty(selectedProfile.PasswordHash)
            )
                return null;

            bool isValid = BCrypt.Net.BCrypt.Verify(password, selectedProfile.PasswordHash);
            return isValid ? selectedProfile : null;
        }
    }
}
