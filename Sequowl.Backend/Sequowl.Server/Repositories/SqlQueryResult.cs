namespace Sequowl.Host.Repositories;

public record SqlQueryResult
{
    public static SqlQueryResult Query(IReadOnlyList<dynamic> data, double timeElapsed)
    {
        return new SqlQueryResult()
        {
            Data = new Optional<IReadOnlyList<dynamic>>(data),
            TimeElapsed = timeElapsed,
            IsSucess = true,
        };
    }

    public static SqlQueryResult Altered(int rowsAffected, double timeElapsed)
    {
        return new SqlQueryResult
        {
            TimeElapsed = timeElapsed,
            IsSucess = true,
            RowsAffected = Optional<string>.For($"{rowsAffected} have been affected.")
        };
    }

    public static SqlQueryResult Failure(string error, double timeElapsed)
    {
        return new SqlQueryResult
        {
            Error = Optional<string>.For(error),
            TimeElapsed = timeElapsed,
            IsSucess = false
        };
    }


    public Optional<IReadOnlyList<dynamic>> Data { get; private set; } = new(null);
    public Optional<string> Error { get; private set; } = new(null);
    public Optional<string> RowsAffected { get; private set; } = new(null);
    public double TimeElapsed { get; private set; }
    public bool IsSucess { get; private set; }
}