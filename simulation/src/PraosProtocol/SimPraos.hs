{-# LANGUAGE DataKinds #-}
{-# LANGUAGE FlexibleContexts #-}
{-# LANGUAGE FlexibleInstances #-}
{-# LANGUAGE GADTs #-}
{-# LANGUAGE PolyKinds #-}
{-# LANGUAGE RankNTypes #-}
{-# LANGUAGE ScopedTypeVariables #-}

module PraosProtocol.SimPraos where

import Chan (ConnectionConfig, mkConnectionConfig, newConnectionBundle)
import Chan.TCP
import Control.Monad.Class.MonadAsync (MonadAsync (..))
import Control.Monad.IOSim as IOSim (IOSim, runSimTrace)
import Control.Tracer as Tracer (
  Contravariant (..),
  Tracer,
  traceWith,
 )
import qualified Data.ByteString as BS
import Data.Function (fix)
import Data.Map.Strict (Map)
import qualified Data.Map.Strict as Map
import Data.Set (Set)
import qualified Data.Set as Set
import P2P
import PraosProtocol.Common hiding (Point)
import PraosProtocol.PraosNode (PraosMessage, runPraosNode)
import SimTCPLinks
import SimTypes

type PraosTrace = [(Time, PraosEvent)]

data PraosEvent
  = -- | Declare the nodes and links
    PraosEventSetup
      !World
      !(Map NodeId Point) -- nodes and locations
      !(Set Link) -- links between nodes
  | -- | An event at a node
    PraosEventNode (LabelNode (PraosNodeEvent BlockBody))
  | -- | An event on a tcp link between two nodes
    PraosEventTcp (LabelLink (TcpEvent (PraosMessage BlockBody)))
  deriving (Show)

exampleTrace1 :: PraosTrace
exampleTrace1 = traceRelayLink1 $ mkConnectionConfig True True 0.1 (Just 1000000)

traceRelayLink1 ::
  ConnectionConfig ->
  PraosTrace
traceRelayLink1 connectionConfig =
  selectTimedEvents $
    runSimTrace $ do
      traceWith tracer $
        PraosEventSetup
          World
            { worldDimensions = (500, 500)
            , worldShape = Rectangle
            }
          ( Map.fromList
              [ (nodeA, Point 50 100)
              , (nodeB, Point 450 100)
              ]
          )
          ( Set.fromList
              [(nodeA :<- nodeB), (nodeB :<- nodeA)]
          )
      let praosConfig = defaultPraosConfig
      let chainA =
            mkChainSimple praosConfig.headerSize $
              [ fix $ \b -> BlockBody (BS.singleton word) (praosConfig.bodySize b)
              | word <- [0 .. 9]
              ]
      let chainB = Genesis
      (pA, cB) <- newConnectionBundle (praosTracer nodeA nodeB) connectionConfig
      (cA, pB) <- newConnectionBundle (praosTracer nodeA nodeB) connectionConfig
      concurrently_
        (runPraosNode (nodeTracer nodeA) praosConfig chainA [pA] [cA])
        (runPraosNode (nodeTracer nodeB) praosConfig chainB [pB] [cB])
      return ()
 where
  (nodeA, nodeB) = (NodeId 0, NodeId 1)

  tracer :: Tracer (IOSim s) PraosEvent
  tracer = simTracer

  nodeTracer :: NodeId -> Tracer (IOSim s) (PraosNodeEvent BlockBody)
  nodeTracer n =
    contramap (PraosEventNode . LabelNode n) tracer

  praosTracer ::
    NodeId ->
    NodeId ->
    Tracer (IOSim s) (LabelTcpDir (TcpEvent (PraosMessage BlockBody)))
  praosTracer nfrom nto =
    contramap (PraosEventTcp . labelDirToLabelLink nfrom nto) tracer
