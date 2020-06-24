# GRPC status codes

Generally each service will handle status codes according to the following lines of thinking. It is considered a bug for a service to use a status code in a way that conflicts with this document or conflicts with the [standard usage](https://grpc.github.io/grpc/core/md_doc_statuscodes.html) of the GRPC status codes.


### Client Errors

* INVALID_ARGUMENT
The service cannot accept the request with the arguments that were provided, without regard for temporality (ie not due to the data that is present/not present on the server, but rather that 100% of correctly written implementations will fail if those arguments are passed again)

* OUT_OF_RANGE
An ordering or length invariant would be violated in order to serve the request (eg providing a truncated checksum)


### Server Errors

* FAILED_PRECONDITION
The system cannot process the value, but through no direct fault of the client, generally the casual chain will have to be reinitiated in order for this request to succeed (eg asking for the status of job identifiers that are not present on an executor)

* ABORTED
Some higher-level construct needs to be restarted in order for this call to succeed (eg asking for the results of a job that failed)

* UNAVAILABLE
The server spilled some milk, retry at the next convenient opportunity

