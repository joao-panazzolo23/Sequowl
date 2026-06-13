using Grpc.Core;
using Mediator;
using Sequowl.Application.Queries.Commands;
using Sequowl.Contracts;
using Sequowl.Host.Results;

namespace Sequowl.Host.Services;

/// <summary>
/// service ExecuteQuery{
/// rpc ExecuteQuery(QueryCommand) returns (QueryResult);
/// }
/// </summary>
public class QueryExecuter(
    IMediator mediator
) : ExecuteQuery.ExecuteQueryBase
{
    public override async Task<QueryResult> ExecuteQuery(
        QueryRequest request,
        ServerCallContext context)
    {
        var options = new DatabaseOptions(
            SqlCommand: request.Sql,
            Host: request.Host,
            Database: request.Database,
            User: request.Username,
            Password: request.Password,
            Schema: request.Schema
        );

        var command = new ConnectToDatabaseCommand(
            Options: options,
            ConnectionString: request.ConnectionString,
            Provider: request.Provider
        );

        var result = await mediator.Send(command);

        return new QueryResult()
        {
                
        };
    }
}

public record ConnectToDatabaseCommand(
    DatabaseOptions Options,
    string ConnectionString,
    string Provider
) : ICommand<Result<QueryExecutionInfo>>;

public record QueryExecutionInfo
{
    public IEnumerable<dynamic> Data { get; private set; }
    public decimal TimeElapsed { get; private set; }
    public string ErrorMessage { get; private set; }
    public string ColumnsAffected { get; private set; }
}