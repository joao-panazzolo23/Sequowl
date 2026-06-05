var builder = WebApplication.CreateBuilder(args);

builder.Services.AddGrpc(o =>
{
    o.EnableDetailedErrors = true;
    o.IgnoreUnknownServices = true;
});

var app = builder.Build();

app.Run();

// gRPC stands for General Remote Procedure Call
// gRPC works with Protocol Buffer/ Protobuf as serialization.
// proto is a contract for gRPC