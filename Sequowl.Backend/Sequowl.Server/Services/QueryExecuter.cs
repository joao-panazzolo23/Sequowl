using Grpc.Core;

namespace Sequowl.Host.Services;

/// <summary>
/// service ExecuteQuery{
/// rpc ExecuteQuery(QueryCommand) returns (QueryResult);
/// }
/// </summary>
public class QueryExecuter : ExecuteQuery.ExecuteQueryBase
{
    public override Task<QueryResult> ExecuteQuery(
        QueryCommand request,
        ServerCallContext context)
    {
        return Task.FromResult(new QueryResult());
    }
}