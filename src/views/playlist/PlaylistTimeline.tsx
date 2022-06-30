import React from 'react'
import { invoke } from '@tauri-apps/api'
import styles from './styles.module.scss'

const zeroPad = (num: number, places: number): string => (num.toString()).padStart(places, '0')

const PlaylistTimeline = () => {
  const [limit, setLimit] = React.useState<number>(16)
  const [ratio, setRatio] = React.useState<number>(1)
  const range = [...Array(limit - 1).keys()].map(e => e + 1)

  React.useEffect(() => {
    const getPlaylistTimeline = async () => {
      const [maxPlaylistBeats, maxBeatsDisplayed, displayRatio] = 
        await invoke<[number, number, number]>('get_playlist_timeline', {})
      setLimit(maxPlaylistBeats)
      setRatio(displayRatio)
    }

    getPlaylistTimeline()
  }, [])

  return (
    <div className={`${styles.PlaylistTimelineContainer}`}>
      <div 
        style={{ width: `${100 * ratio}%` }}
        className={`${styles.PlaylistTimeline} bg-slate-300`}
      >
        {range.map(e => (
          <div 
            key={`timeline-segment-${e}`}
            id={`playlist-timeline-segment-${e}`}
            style={{ width: `${1 / limit}%` }}
            className={`${styles.PlaylistTimelineSegment} border-l-slate-400`}
          >
            <span className={`${styles.PlaylistTimelineSegmentText} bg-slate-300 text-gray-600 text-xs font-mono`}>
              {e < 10 ? zeroPad(e, 1) : e}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}

export default PlaylistTimeline
