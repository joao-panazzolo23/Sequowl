namespace Sequowl.Host.Repositories;

public record SqlQueryResult
{
    public static SqlQueryResult Query(IReadOnlyList<dynamic> data, double timeElapsed)
    {
        return new SqlQueryResult
        {
            Data = Optional<IReadOnlyList<dynamic>?>.For(data),
            TimeElapsed = timeElapsed,
            IsSuccess = true,
        };
    }

    public static SqlQueryResult Altered(int rowsAffected, double timeElapsed)
    {
        return new SqlQueryResult
        {
            TimeElapsed = timeElapsed,
            IsSuccess = true,
            RowsAffected = Optional<string?>.For($"{rowsAffected} have been affected.")
        };
    }

    public static SqlQueryResult Failure(string error, double timeElapsed)
    {
        return new SqlQueryResult
        {
            Error = Optional<string?>.For(error),
            TimeElapsed = timeElapsed,
            IsSuccess = false
        };
    }


    public Optional<IReadOnlyList<dynamic>?> Data { get; private set; } = new(null);
    public Optional<string?> Error { get; private set; } = new(null);
    public Optional<string?> RowsAffected { get; private set; } = new(null);
    public double TimeElapsed { get; private set; }
    public bool IsSuccess { get; private set; }
}