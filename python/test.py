from server_pb2_grpc import MediaControllerStub
import grpc
from server_pb2 import SlideAddRequest

channel = grpc.insecure_channel('localhost:50051')
stub = MediaControllerStub(channel)
slide_add_request = SlideAddRequest()
slide_add_request.slide_name = "asd"
slide_add_request.text = "Hello Jan"
stub.AddSlide(slide_add_request)
