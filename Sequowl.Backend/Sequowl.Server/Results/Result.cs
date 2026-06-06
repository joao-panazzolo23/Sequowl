namespace Sequowl.Host.Results;

public class Result<T>
{
    private Result(T? value = default, string error = "")
    {
        Value = value;
        IsSuccess = true;
        Error = error;
    }

    public T? Value { get; }
    public string? Error { get; }
    public bool IsSuccess { get; }

    public static Result<T> Ok(T value) => new(value: value);
    public static Result<T> Fail(string error) => new(error: error);
}