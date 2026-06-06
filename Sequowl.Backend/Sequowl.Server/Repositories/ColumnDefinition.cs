namespace Sequowl.Host.Repositories;

//I don't remember what this used to do.
public sealed record ColumnDefinition(string Name, Type ClrType, string DbTypeName);