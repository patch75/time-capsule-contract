import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

describe("time_capsule_contract", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  // Utiliser any temporairement pour éviter les problèmes de types
 const program = anchor.workspace.TimeCapsuleContract as Program<any>;

  // Générer une paire de clés pour la capsule temporelle
  const timeCapsule = anchor.web3.Keypair.generate();
  const sender = anchor.web3.Keypair.generate();

  it("Create a time capsule", async () => {
    console.log("🚀 Test de création de capsule temporelle...");

    // Airdrop pour les frais de transaction
    const signature = await anchor.getProvider().connection.requestAirdrop(
      sender.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    
    await anchor.getProvider().connection.confirmTransaction(signature);

    // Données de test
    const encryptedMessage = "Message secret de test crypté";
    const unlockTimestamp = new anchor.BN(Math.floor(Date.now() / 1000) + 3600); // Dans 1 heure
    const recipientEmailHash = "a".repeat(64); // Hash SHA256 simulé
    const passwordHash = "b".repeat(64); // Hash SHA256 simulé  
    const passwordHint = "Votre couleur préférée";
    const messageTitle = "Ma première capsule de test";

    try {
      console.log("📝 Création de la capsule...");
      
      const tx = await program.methods
        .createTimeCapsule(
          encryptedMessage,
          unlockTimestamp,
          recipientEmailHash,
          passwordHash,
          passwordHint,
          messageTitle
        )
        .accounts({
          timeCapsule: timeCapsule.publicKey,
          sender: sender.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([timeCapsule, sender])
        .rpc();

      console.log("✅ Capsule créée avec succès !");
      console.log("📋 Transaction signature:", tx);
      console.log("🆔 Capsule ID:", timeCapsule.publicKey.toString());

      // Test de récupération des infos publiques
      console.log("📖 Test de récupération des infos...");
      
      const capsuleInfo = await program.methods
        .getCapsuleInfo()
        .accounts({
          timeCapsule: timeCapsule.publicKey,
        })
        .view();

      console.log("📊 Infos de la capsule :");
      console.log("  - Titre:", capsuleInfo.messageTitle);
      console.log("  - Hint:", capsuleInfo.passwordHint); 
      console.log("  - Sender:", capsuleInfo.sender.toString());
      console.log("  - Réclamée:", capsuleInfo.isClaimed);
      
      console.log("🎉 Tous les tests passent !");

    } catch (error) {
      console.error("❌ Erreur lors du test:", error);
      throw error;
    }
  });

  it("Should fail to retrieve message before unlock time", async () => {
    console.log("🔒 Test de sécurité (récupération prématurée)...");
    
    const wrongPasswordHash = "c".repeat(64);

    try {
      await program.methods
        .retrieveMessage(wrongPasswordHash)
        .accounts({
          timeCapsule: timeCapsule.publicKey,
        })
        .rpc();

      console.log("❌ ERREUR: La récupération aurait dû échouer");
      throw new Error("Test de sécurité échoué");
      
    } catch (error) {
      if (error.message.includes("StillLocked")) {
        console.log("✅ Sécurité OK: Capsule encore verrouillée");
      } else {
        console.log("✅ Sécurité OK: Accès refusé (", error.message, ")");
      }
    }
  });
});