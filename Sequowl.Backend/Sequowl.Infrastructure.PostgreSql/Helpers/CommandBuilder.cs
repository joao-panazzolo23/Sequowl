using Npgsql;
using Sequowl.Infrastructure.PostgreSql.Connection;

namespace Sequowl.Infrastructure.PostgreSql.Helpers;

public sealed class CommandBuilder(PostgreSqlSession session)
{
    public NpgsqlCommand Build(string sql)
    {
        var cmd = session.Connection.CreateCommand();
        cmd.CommandText = sql;

        if (session.Transaction is not null)
            cmd.Transaction = session.Transaction;

        return cmd;
    }

    public NpgsqlCommand Build(string sql, Action<NpgsqlParameterCollection> parameters)
    {
        var cmd = Build(sql);
        parameters(cmd.Parameters);
        return cmd;
    }
}