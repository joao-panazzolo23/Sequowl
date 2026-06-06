using Sequowl.Host.Models;

namespace Sequowl.Host.Repositories;

public interface IDatabaseConnection
{
    Task<IReadOnlyList<DatabaseInfo>> GetDatabases(CancellationToken cancellationToken);

    Task<IReadOnlyList<TableInfo>> GetObjects(
        string databaseName,
        CancellationToken cancellationToken
    );

    Task<IReadOnlyList<ColumnInfo>> GetColumns(
        string databaseName,
        string tableName,
        CancellationToken cancellationToken
    );

    Task<SqlQueryResult> ExecuteQuery(
        string sqlCommand,
        CancellationToken cancellationToken
    );
}