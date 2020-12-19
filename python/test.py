from server_pb2_grpc import MediaControllerStub
import grpc
from server_pb2 import SlideAddRequest, WidgetAddRequest, ShowSlideRequest, Widget

channel = grpc.insecure_channel('localhost:50051')
stub = MediaControllerStub(channel)
slide_add_request = SlideAddRequest()

widget = Widget()
widget.x = 5
widget.y = 5
widget.z = 2
widget.rectangle_widget.color.red = 0.0
widget.rectangle_widget.color.blue = 1.0
widget.rectangle_widget.color.green = 0.5
widget.rectangle_widget.color.alpha = 1.0
widget.rectangle_widget.width = 500
widget.rectangle_widget.height = 300
slide_add_request.widgets.append(widget)

widget = Widget()
widget.x = 10
widget.y = 10
widget.z = 5
#widget.video_widget.path = "/home/jan/Downloads/Biking_Girl_Alpha.mov"
widget.video_widget.path = "/home/jan/src/mpf-mc/mpfmc/tests/machine_files/video/videos/mpf_video_small_test.mp4"
slide_add_request.widgets.append(widget)

new_slide = stub.AddSlide(slide_add_request)
slide_id = new_slide.slide_id

# Slide has been created now lets add more widgets
widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = slide_id

widget = Widget()
widget.x = 50
widget.y = 50
widget.z = 4
widget.image_widget.path = "/home/jan/src/missionpinball-website/images/mpf-flag-tiny.png"
widget_add_request.widgets.append(widget)

widget = Widget()
widget.x = 20
widget.y = 150
widget.z = 5
widget.label_widget.color.red = 1.0
widget.label_widget.color.blue = 0.0
widget.label_widget.color.green = 0.0
widget.label_widget.color.alpha = 1.0
widget.label_widget.text = "Hello World"
widget.label_widget.font_name = "DejaVuSerif.ttf"
widget.label_widget.font_size = 32
widget_add_request.widgets.append(widget)

widget = Widget()
widget.x = 30.0
widget.y = 30.0
widget.z = 6
widget.line_widget.color.red = 1.0
widget.line_widget.color.blue = 0.0
widget.line_widget.color.green = 0.0
widget.line_widget.color.alpha = 1.0
widget.line_widget.x1 = 3.0
widget.line_widget.y1 = 50.0
widget.line_widget.x2 = 150.0
widget.line_widget.y2 = 300.0
widget.line_widget.width = 10.0
widget_add_request.widgets.append(widget)

stub.AddWidgetsToSlide(widget_add_request)

show_slide_request = ShowSlideRequest()
show_slide_request.slide_id = slide_id
stub.ShowSlide(show_slide_request)
