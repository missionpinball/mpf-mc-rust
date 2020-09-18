from server_pb2_grpc import MediaControllerStub
import grpc
from server_pb2 import SlideAddRequest
from server_pb2 import WidgetAddRequest
from server_pb2 import ShowSlideRequest

channel = grpc.insecure_channel('localhost:50051')
stub = MediaControllerStub(channel)
slide_add_request = SlideAddRequest()
new_slide = stub.AddSlide(slide_add_request)

widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = new_slide.slide_id
widget_add_request.text = "Hello Mausi <3"
stub.AddWidgetsToSlide(widget_add_request)

show_slide_request = ShowSlideRequest()
show_slide_request.slide_id = new_slide.slide_id
stub.ShowSlide(show_slide_request)
