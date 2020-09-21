from server_pb2_grpc import MediaControllerStub
import grpc
from server_pb2 import SlideAddRequest, WidgetAddRequest, ShowSlideRequest

channel = grpc.insecure_channel('localhost:50051')
stub = MediaControllerStub(channel)
slide_add_request = SlideAddRequest()
new_slide = stub.AddSlide(slide_add_request)

widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = new_slide.slide_id
widget_add_request.x = 10
widget_add_request.y = 50
widget_add_request.z = 2
widget_add_request.color.red = 0.0
widget_add_request.color.blue = 1.0
widget_add_request.color.green = 0.0
widget_add_request.color.alpha = 1.0
widget_add_request.rectangle_widget.width = 300
widget_add_request.rectangle_widget.height = 100
stub.AddWidgetsToSlide(widget_add_request)

widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = new_slide.slide_id
widget_add_request.x = 20
widget_add_request.y = 100
widget_add_request.z = 4
widget_add_request.color.red = 1.0
widget_add_request.color.blue = 0.0
widget_add_request.color.green = 0.0
widget_add_request.color.alpha = 1.0
widget_add_request.text_widget.text = "Hello Mausi <3"
stub.AddWidgetsToSlide(widget_add_request)

widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = new_slide.slide_id
widget_add_request.x = 100
widget_add_request.y = 100
widget_add_request.z = 4
widget_add_request.color.red = 1.0
widget_add_request.color.blue = 0.0
widget_add_request.color.green = 0.0
widget_add_request.color.alpha = 1.0
widget_add_request.image_widget.width = 100
widget_add_request.image_widget.height = 200
widget_add_request.image_widget.path = "/home/jan/src/missionpinball-website/images/mpf-flag-tiny.png"
stub.AddWidgetsToSlide(widget_add_request)

show_slide_request = ShowSlideRequest()
show_slide_request.slide_id = new_slide.slide_id
stub.ShowSlide(show_slide_request)
