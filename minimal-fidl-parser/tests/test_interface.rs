mod shared;
use minimal_fidl_parser::{interface, BasicContext, Rules};
use shared::shared;
#[test]
fn test_interface_1() {
    let src = "<** @description: Indicate end of playlist. **>
                    interface endOfPlaylist { }";
    let result = shared(src, interface::<BasicContext>, Rules::interface);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_interface_2() {
    let src = "interface DerivedInterface  {

    }";
    let result = shared(src, interface::<BasicContext>, Rules::interface);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_interface_3() {
    let src = "<** @description : Media playback interface.
    
        This is a synthetic example interface. There might be an implementation
        somewhere, but its main purpose is to demonstrate the various Franca IDL features.
        
        The interface definition uses some basic data types, which are defined
        in a separate Franca IDL file (i.e., CommonTypes.fidl).
    **> 
    interface MediaPlayer {
        version { major 4 minor 2 }
    
        <** @description: Playlist which is currently active. **>
        attribute Playlist currentPlaylist
    
        <** @description: Track which is currently playing. **>
        attribute TrackId currentTrack
        
        <** @description: Remaining duration of current track. **>
        attribute Duration remainingTrack
        
        <** @description: Remaining duration of current playlist. **>
        attribute Duration remainingAll
        
        <** @description: Track which is currently playing. **>
        attribute RepeatMode mode
            
        <** @description: Get the metadata of any track, given its id. **>
        method getMetadata {
            in {
                TrackId trackId
            }
            out {
                <** @description: A struct containing the metadata. **>
                TrackInfo info
            }
        }
    
        <** @description: Clear the playlist. **>
        method clear { }	
    
        <** @description: Append another track to the playlist. **>
        method appendTrack {
            in {
                TrackId trackId
            }
        }
        
        <** @description: Start playing the current playlist or resume playing
                          after pause(). **>
        method play { }
    
        <** @description: Pause playing the current playlist. **>
        method pause { }
        
        <** @description: Set current repeat mode. **>
        method setRepeatMode {
            in {
                RepeatMode mode
            }
        }
        
        <** @description: Switch to the next track (if any). **>
        method nextTrack { } 
    
        <** @description: Switch to the previous track (if any). **>
        method previousTrack { } 
    
    
    
        // *** data types
    
        <** @description : Repeat modes for playback. **>
        enumeration RepeatMode {
            MODE_REPEAT_NONE   = 0
            MODE_REPEAT_SINGLE = 1
            MODE_REPEAT_ALL    = 2
        }
    
        <** @description: Track metadata. **>
        struct TrackInfo {
            String title
            String album
            String interpret
            String composer
            String genre
            Year year
            Duration trackLength
        }
    
    }";
    let result = shared(src, interface::<BasicContext>, Rules::interface);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_interface_4() {
    let src = "<** @description  : Bluetooth Manager interface.
    
                        As this is an example interface only, it doesn't contain any
                        more documentation. It is just a very simple interface definition.
                        
        @source_alias : derived from org.blueman.Applet **>
    interface BluetoothManager {
        version {
            major 0
            minor 1		
        }
    
        method RefreshServices {
            in {
                String path
            }
        }
    
        method ServiceProxy {
            in {
                String interface
                String object_path
                String _method
                String[] args
            }
        }
    
        method SetPluginConfig {
            in {
                String plugin
                Boolean value
            }
        }
        
        method CreateDevice {
            in {
                String adapter_path
                String address
                Boolean pair
                UInt32 time
            }
        }
    
        method QueryAvailablePlugins {
            out {
                String[] _outArg0
            }
        }
    
        method DhcpClient {
            in {
                String interface
            }
        }
    
        method TransferControl {
            in {
                String pattern
                String action
            }
        }
    
        method GetBluetoothStatus {
            out {
                Boolean _outArg0
            }
        }
    
        method CancelDeviceCreation {
            in {
                String adapter_path
                String address
            }
        }
    
        method DisconnectDevice {
            in {
                String obj_path
            }
        }
    
        method RfcommDisconnect {
            in {
                String device
                String rfdevice
            }
        }
    
        method SetBluetoothStatus {
            in {
                Boolean status
            }
        }
    
        method QueryPlugins {
            out {
                String[] _outArg0
            }
        }
    
        method TransferStatus {
            in {
                String pattern
            }
            out {
                Int32 _outArg0
            }
        }
    
        method SetTimeHint {
            in {
                UInt32 time
            }
        }
    
        method RfcommConnect {
            in {
                String device
                String uuid
            }
            out {
                String _outArg0
            }
        }
    

    }";
    let result = shared(src, interface::<BasicContext>, Rules::interface);
    assert_eq!(result, (true, src.len() as u32));
}

#[test]
fn test_interface_5() {
    let src = "interface name {
        enumeration aEnum {
            A = 3
            B = 0x004000
            C = 0b0101001
            D
            E = 10
        }
    }";
    let result = shared(src, interface::<BasicContext>, Rules::interface);
    assert_eq!(result, (true, src.len() as u32));
}
