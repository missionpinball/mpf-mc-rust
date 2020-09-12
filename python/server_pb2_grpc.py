# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc

import server_pb2 as server__pb2


class MediaControllerStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.AddSlide = channel.unary_unary(
                '/mpf.MediaController/AddSlide',
                request_serializer=server__pb2.SlideAddRequest.SerializeToString,
                response_deserializer=server__pb2.SlideAddResponse.FromString,
                )


class MediaControllerServicer(object):
    """Missing associated documentation comment in .proto file."""

    def AddSlide(self, request, context):
        """rpc ConfigureDisplays() returns ();
        rpc RemoveSlide() returns ();
        rpc AddWidgetsToSlide() returns ();
        rpc RemoveWidgetFromSlide() returns ();
        rpc ShowSlide() returns ();
        rpc PreloadAsset() returns ();
        """
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_MediaControllerServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'AddSlide': grpc.unary_unary_rpc_method_handler(
                    servicer.AddSlide,
                    request_deserializer=server__pb2.SlideAddRequest.FromString,
                    response_serializer=server__pb2.SlideAddResponse.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'mpf.MediaController', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class MediaController(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def AddSlide(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/mpf.MediaController/AddSlide',
            server__pb2.SlideAddRequest.SerializeToString,
            server__pb2.SlideAddResponse.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)
