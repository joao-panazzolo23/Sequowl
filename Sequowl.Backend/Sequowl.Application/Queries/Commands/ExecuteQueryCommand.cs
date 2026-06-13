namespace Sequowl.Application.Queries.Commands;

public record DatabaseOptions(
    string SqlCommand,
    string Host,
    string Database,
    string User,
    string Password,
    string Schema
);
